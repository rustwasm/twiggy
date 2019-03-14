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
    let max_items = opts.max_items() as usize;

    // Given a set of items, create a HashMap of the items' names and sizes.
    fn get_names_and_sizes(items: &ir::Items) -> HashMap<String, i64> {
        items
            .iter()
            .map(|item| (item.name(), i64::from(item.size())))
            .collect()
    }

    // Collect the names and sizes of the items in the old and new collections.
    let old_sizes = get_names_and_sizes(old_items);
    let new_sizes = get_names_and_sizes(new_items);

    // Given an item name, create a `DiffEntry` object representing the
    // change in size, or an error if the name could not be found in
    // either of the item collections.
    let get_item_delta = |name: String| -> Result<DiffEntry, traits::Error> {
        let old_size = old_sizes.get::<str>(&name);
        let new_size = new_sizes.get::<str>(&name);
        let delta: i64 = match (old_size, new_size) {
            (Some(old_size), Some(new_size)) => new_size - old_size,
            (Some(old_size), None) => -old_size,
            (None, Some(new_size)) => *new_size,
            (None, None) => {
                return Err(traits::Error::with_msg(format!(
                    "Could not find item with name `{}`",
                    name
                )));
            }
        };
        Ok(DiffEntry { name, delta })
    };

    // Given a result returned by `get_item_delta`, return false if the result
    // represents an unchanged item. Ignore errors, these are handled separately.
    let unchanged_items_filter = |res: &Result<DiffEntry, traits::Error>| -> bool {
        if let Ok(DiffEntry { delta: 0, .. }) = res {
            false
        } else {
            true
        }
    };

    // Create a set of item names from the new and old item collections.
    let names = old_sizes
        .keys()
        .chain(new_sizes.keys())
        .map(|k| k.to_string());

    // If arguments were given to the command, we should filter out items that
    // do not match any of the given names or expressions.
    let names: HashSet<String> = if !opts.items().is_empty() {
        if opts.using_regexps() {
            let regexps = regex::RegexSet::new(opts.items())?;
            names.filter(|name| regexps.is_match(name)).collect()
        } else {
            let item_names = opts.items().iter().collect::<HashSet<_>>();
            names.filter(|name| item_names.contains(&name)).collect()
        }
    } else {
        names.collect()
    };

    // Iterate through the set of item names, and use the closure above to map
    // each item into a `DiffEntry` object. Then, sort the collection.
    let mut deltas = names
        .into_iter()
        .map(get_item_delta)
        .filter(unchanged_items_filter)
        .collect::<Result<Vec<_>, traits::Error>>()?;
    deltas.sort();

    // Create an entry to summarize the diff rows that will be truncated.
    let (rem_cnt, rem_delta): (u32, i64) = deltas
        .iter()
        .skip(max_items)
        .fold((0, 0), |(cnt, rem_delta), DiffEntry { delta, .. }| {
            (cnt + 1, rem_delta + delta)
        });
    let remaining = DiffEntry {
        name: format!("... and {} more.", rem_cnt),
        delta: rem_delta,
    };

    // Create a `DiffEntry` representing the net change, and total row count.
    // If specifying arguments were not given, calculate the total net changes,
    // otherwise find the total values only for items in the the deltas collection.
    let (total_cnt, total_delta) = if opts.items().is_empty() {
        (
            deltas.len(),
            i64::from(new_items.size()) - i64::from(old_items.size()),
        )
    } else {
        deltas
            .iter()
            .fold((0, 0), |(cnt, rem_delta), DiffEntry { delta, .. }| {
                (cnt + 1, rem_delta + delta)
            })
    };
    let total = DiffEntry {
        name: format!("Σ [{} Total Rows]", total_cnt),
        delta: total_delta,
    };

    // Now that the 'remaining' and 'total' summary entries have been created,
    // truncate the vector of deltas before we box up the result, and push
    // the remaining and total rows to the deltas vector.
    deltas.truncate(max_items);
    if rem_cnt > 0 {
        deltas.push(remaining);
    }
    deltas.push(total);

    // Return the results so that they can be emitted.
    let diff = Diff { deltas };
    Ok(Box::new(diff) as Box<traits::Emit>)
}
