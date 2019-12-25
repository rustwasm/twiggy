use std::cmp;
use std::collections::{BTreeMap, BTreeSet};

use twiggy_ir as ir;
use twiggy_opt as opt;
use twiggy_traits as traits;

mod emit;
mod entry;

use self::entry::MonosEntry;

#[derive(Debug)]
struct Monos {
    monos: Vec<MonosEntry>,
}

/// Type alias used to represent a map of generic function names and instantiations.
type MonosMap<'a> = BTreeMap<&'a str, Vec<(String, u32)>>;

/// Collect the monomorphizations of generic functions into a map, then
/// process the entries and sort the resulting vector.
fn collect_monomorphizations<'a>(items: &'a ir::Items) -> MonosMap {
    let unsorted_monos: BTreeMap<&'a str, BTreeSet<(String, u32)>> = items
        .iter()
        .filter_map(|item| {
            if let Some(generic) = item.monomorphization_of() {
                Some((generic, item))
            } else {
                None
            }
        })
        .fold(BTreeMap::new(), |mut monos, (generic, inst)| {
            monos
                .entry(generic)
                .or_insert_with(BTreeSet::new)
                .insert((inst.name().to_string(), inst.size()));
            monos
        });

    unsorted_monos
        .into_iter()
        .map(|(generic, inst_set)| {
            let mut insts = inst_set.into_iter().collect::<Vec<_>>();
            insts.sort_by(|(a_name, a_size), (b_name, b_size)| {
                b_size.cmp(a_size).then(a_name.cmp(b_name))
            });
            (generic, insts)
        })
        .collect()
}

/// Helper function usedd to summarize a sequence of `MonosEntry` objects.
/// Returns a tuple representing the number of items summarized, the total
/// size of the items, and the total approximate potential savings.
fn summarize_entries<'a>(entries: impl Iterator<Item = &'a MonosEntry>) -> (usize, u32, u32) {
    entries.fold(
        (0, 0, 0),
        |(total_cnt, total_size, total_savings),
         MonosEntry {
             insts, size, bloat, ..
         }| {
            (
                total_cnt + 1 + insts.len(),
                total_size + size,
                total_savings + bloat,
            )
        },
    )
}

/// Helper function used to summarize a sequence of tuples representing
/// instantiations of a generic function. Returns a tuple representing the
/// number of instantiations found, and the total size.
fn summarize_insts<'a>(entries: impl Iterator<Item = &'a (String, u32)>) -> (u32, u32) {
    entries.fold((0, 0), |(total_cnt, total_size), (_, size)| {
        (total_cnt + 1, total_size + size)
    })
}

/// Find the approximate potential savings by calculating the benefits of
/// removing the largest instantiation, and the benefits of removing an
/// average instantiation. Returns a tuple containing total size, and bloat.
fn calculate_total_and_bloat(insts: &[(String, u32)]) -> Option<(u32, u32)> {
    if let Some(max) = insts.iter().map(|(_, size)| size).max() {
        let total_size = insts.iter().map(|(_, size)| size).sum::<u32>();
        let inst_cnt = insts.len() as u32;
        let size_per_inst = total_size / inst_cnt;
        let avg_savings = size_per_inst * (inst_cnt - 1);
        let removing_largest_savings = total_size - max;
        let approx_potential_savings = cmp::min(avg_savings, removing_largest_savings);
        Some((total_size, approx_potential_savings))
    } else {
        None
    }
}

/// Process all of the monorphizations, into a vector of `MonosEntry` objects.
fn process_monomorphizations(monos_map: MonosMap, opts: &opt::Monos) -> Vec<MonosEntry> {
    let mut monos = monos_map
        .into_iter()
        .filter_map(|(g, insts)| {
            calculate_total_and_bloat(&insts).map(|(total, bloat)| (g, insts, total, bloat))
        })
        .map(|(g, mut insts, t, b)| {
            // Truncate `insts` according to the relevant options before
            // we map these values into `MonosEntry` objects.
            if opts.only_generics() {
                insts.truncate(0);
            } else {
                let max_monos = opts.max_monos() as usize;
                let (rem_cnt, rem_size) = summarize_insts(insts.iter().skip(max_monos));
                insts.truncate(max_monos);
                if rem_cnt > 0 {
                    insts.push((format!("... and {} more.", rem_cnt), rem_size));
                }
            };
            (g, insts, t, b)
        })
        .map(|(name, insts, size, bloat)| MonosEntry {
            name: name.to_string(),
            insts,
            size,
            bloat,
        })
        .collect::<Vec<_>>();
    monos.sort();
    monos
}

/// Adds entries to summarize remaining rows that will be truncated, and
/// totals for the entire set of monomorphizations.
fn add_stats(mut monos: Vec<MonosEntry>, opts: &opt::Monos) -> Vec<MonosEntry> {
    let max_generics = opts.max_generics() as usize;

    // Create an entry to represent the remaining rows that will be truncated.
    let (rem_cnt, rem_size, rem_savings) = summarize_entries(monos.iter().skip(max_generics));
    let remaining = MonosEntry {
        name: format!("... and {} more.", rem_cnt),
        size: rem_size,
        insts: vec![],
        bloat: rem_savings,
    };

    // Create an entry to represent the 'total' summary.
    let (total_cnt, total_size, total_savings) = summarize_entries(monos.iter());
    let total = MonosEntry {
        name: format!("Σ [{} Total Rows]", total_cnt),
        size: total_size,
        insts: vec![],
        bloat: total_savings,
    };

    // Truncate the vector, and add the 'remaining' and 'total' summary entries.
    monos.truncate(max_generics);
    if rem_cnt > 0 {
        monos.push(remaining);
    }
    monos.push(total);
    monos
}

/// Find bloaty monomorphizations of generic functions.
pub fn monos(
    items: &mut ir::Items,
    opts: &opt::Monos,
) -> Result<Box<dyn traits::Emit>, traits::Error> {
    let monos_map = collect_monomorphizations(&items);
    let mut monos = process_monomorphizations(monos_map, &opts);
    monos = add_stats(monos, &opts);
    Ok(Box::new(Monos { monos }) as Box<_>)
}
