use std::collections::{HashMap, HashSet};

use regex;

use twiggy_ir as ir;
use twiggy_opt as opt;
use twiggy_traits as traits;

mod emit;
mod entry;

use self::entry::DiffEntry;

#[derive(Debug)]
struct Diff {
    deltas: Vec<DiffEntry>,
}

/// Compute the diff between two sets of items.
pub fn diff(
    old_items: &mut ir::Items,
    new_items: &mut ir::Items,
    opts: &opt::Diff,
) -> Result<Box<traits::Emit>, traits::Error> {
    let old_items_by_name = collect_items_by_name(old_items);
    let new_items_by_name = collect_items_by_name(new_items);
    let names = get_unique_names(old_items_by_name.keys(), new_items_by_name.keys(), opts)?;
    let entries = collect_entries(names, old_items_by_name, new_items_by_name)?;
    let summary = summarize_entries(&entries, old_items.size(), new_items.size(), opts);

    // Truncate the entries and attach the summary rows.
    let deltas = entries
        .into_iter()
        .take(opts.max_items() as usize)
        .chain(summary)
        .collect();

    // Return the results so that they can be emitted.
    let diff = Diff { deltas };
    Ok(Box::new(diff) as Box<traits::Emit>)
}

// Given a set of items, create a HashMap of the items' names and sizes.
fn collect_items_by_name(items: &ir::Items) -> HashMap<String, &ir::Item> {
    items.iter().map(|item| (item.name(), item)).collect()
}

// Create a set of unique item names in the new and old item collections.
// If arguments were given to the command, filter out items that do not match
// any of the given names or expressions.
fn get_unique_names<T, I>(
    old_names: T,
    new_names: T,
    opts: &opt::Diff,
) -> Result<HashSet<String>, traits::Error>
where
    T: Iterator<Item = I>,
    I: ToString,
{
    let names_iter = old_names.chain(new_names).map(|k| k.to_string());
    let names: HashSet<String> = if !opts.items().is_empty() {
        if opts.using_regexps() {
            let regexps = regex::RegexSet::new(opts.items())?;
            names_iter.filter(|name| regexps.is_match(name)).collect()
        } else {
            let item_names = opts.items().iter().collect::<HashSet<_>>();
            names_iter
                .filter(|name| item_names.contains(&name))
                .collect()
        }
    } else {
        names_iter.collect()
    };
    Ok(names)
}

// Iterate through the set of item names, and use the closure above to map
// each item into a `DiffEntry` object. Then, sort the collection.
fn collect_entries(
    names: impl IntoIterator<Item = String>,
    old_sizes: HashMap<String, &ir::Item>,
    new_sizes: HashMap<String, &ir::Item>,
) -> Result<Vec<DiffEntry>, traits::Error> {
    let mut deltas = names
        .into_iter()
        .map(|name| {
            let old_item = old_sizes.get::<str>(&name);
            let new_item = new_sizes.get::<str>(&name);
            (old_item, new_item)
        })
        .map(|(old_item, new_item)| get_item_delta(old_item, new_item))
        .filter(|entry| {
            if let Ok(DiffEntry { delta: 0, .. }) = entry {
                false
            } else {
                true
            }
        })
        .collect::<Result<Vec<_>, traits::Error>>()?;
    deltas.sort();
    Ok(deltas)
}

// Given an item name, create a `DiffEntry` object representing the
// change in size, or an error if the name could not be found in
// either of the item collections.
fn get_item_delta(
    old_item: Option<&&ir::Item>,
    new_item: Option<&&ir::Item>,
) -> Result<DiffEntry, traits::Error> {
    match (old_item, new_item) {
        (Some(old_item), Some(new_item)) => Ok(DiffEntry {
            delta: i64::from(new_item.size()) - i64::from(old_item.size()),
            name: new_item.decorated_name(),
        }),
        (Some(old_item), None) => Ok(DiffEntry {
            delta: -i64::from(old_item.size()),
            name: old_item.decorated_name(),
        }),
        (None, Some(new_item)) => Ok(DiffEntry {
            delta: i64::from(new_item.size()),
            name: new_item.decorated_name(),
        }),
        (None, None) => Err(traits::Error::with_msg("Unexpected item name found")),
    }
}

/// Returns an iterator of DiffEntry objects summarizing the deltas.
fn summarize_entries<T>(
    deltas: &[DiffEntry],
    old_total_size: T,
    new_total_size: T,
    opts: &opt::Diff,
) -> impl IntoIterator<Item = DiffEntry>
where
    i64: std::convert::From<T>,
{
    let total = total_summary(deltas, old_total_size, new_total_size, opts);
    if let Some(remaining) = remaining_summary(deltas, opts) {
        vec![remaining, total]
    } else {
        vec![total]
    }
}

// Create a `DiffEntry` representing the net change, and total row count.
// If specifying arguments were not given, calculate the total net changes,
// otherwise find the total values only for items in the the deltas collection.
fn total_summary<T>(
    deltas: &[DiffEntry],
    old_total_size: T,
    new_total_size: T,
    opts: &opt::Diff,
) -> DiffEntry
where
    i64: std::convert::From<T>,
{
    let (total_cnt, total_delta) = if opts.items().is_empty() {
        (
            deltas.len(),
            i64::from(new_total_size) - i64::from(old_total_size),
        )
    } else {
        deltas
            .iter()
            .fold((0, 0), |(cnt, rem_delta), DiffEntry { delta, .. }| {
                (cnt + 1, rem_delta + delta)
            })
    };
    DiffEntry {
        name: format!("Σ [{} Total Rows]", total_cnt),
        delta: total_delta,
    }
}

/// Create an entry to summarize the diff rows that will be truncated.
fn remaining_summary(deltas: &[DiffEntry], opts: &opt::Diff) -> Option<DiffEntry> {
    match deltas
        .iter()
        .skip(opts.max_items() as usize)
        .fold((0, 0), |(rem_cnt, rem_delta), DiffEntry { delta, .. }| {
            (rem_cnt + 1, rem_delta + delta)
        }) {
        (rem_cnt, rem_delta) if rem_cnt > 0 => Some(DiffEntry {
            name: format!("... and {} more.", rem_cnt),
            delta: rem_delta,
        }),
        _ => None,
    }
}
