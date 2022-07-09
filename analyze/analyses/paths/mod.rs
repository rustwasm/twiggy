use std::collections::BTreeSet;

use regex;

use twiggy_ir as ir;
use twiggy_opt as opt;
use twiggy_traits as traits;

mod paths_emit;
mod paths_entry;

use self::paths_entry::PathsEntry;

#[derive(Debug)]
struct Paths {
    opts: opt::Paths,
    entries: Vec<PathsEntry>,
}

/// Find all retaining paths for the given items.
pub fn paths(items: &mut ir::Items, opts: &opt::Paths) -> anyhow::Result<Box<dyn traits::Emit>> {
    // The predecessor tree only needs to be computed if we are ascending
    // through the retaining paths.
    if !opts.descending() {
        items.compute_predecessors();
    }

    // Initialize the collection of Id values whose retaining paths we will emit.
    let opts = opts.clone();
    let entries = get_starting_positions(items, &opts)?
        .iter()
        .map(|id| create_entry(*id, items, &opts, &mut BTreeSet::new()))
        .collect();

    let paths = Paths { opts, entries };

    Ok(Box::new(paths) as Box<_>)
}

/// This helper function is used to collect the `ir::Id` values for the top-most
/// path entries for the `Paths` object, based on the given options.
fn get_starting_positions(items: &ir::Items, opts: &opt::Paths) -> anyhow::Result<Vec<ir::Id>> {
    // Collect Id's if no arguments are given and we are ascending the retaining paths.
    let get_functions_default = || -> Vec<ir::Id> {
        let mut sorted_items = items
            .iter()
            .filter(|item| item.id() != items.meta_root())
            .collect::<Vec<_>>();
        sorted_items.sort_by(|a, b| b.size().cmp(&a.size()));
        sorted_items.iter().map(|item| item.id()).collect()
    };

    // Collect Id's if no arguments are given and we are descending the retaining paths.
    let get_functions_default_desc = || -> Vec<ir::Id> {
        let mut roots = items
            .neighbors(items.meta_root())
            .map(|id| &items[id])
            .collect::<Vec<_>>();
        roots.sort_by(|a, b| b.size().cmp(&a.size()));
        roots.into_iter().map(|item| item.id()).collect()
    };

    // Collect Id's if arguments were given that should be used as regular expressions.
    let get_regexp_matches = || -> anyhow::Result<Vec<ir::Id>> {
        let regexps = regex::RegexSet::new(opts.functions())?;
        let matches = items
            .iter()
            .filter(|item| regexps.is_match(item.name()))
            .map(|item| item.id())
            .collect();
        Ok(matches)
    };

    // Collect Id's if arguments were given that should be used as exact names.
    let get_exact_matches = || -> Vec<ir::Id> {
        opts.functions()
            .iter()
            .filter_map(|s| items.get_item_by_name(s))
            .map(|item| item.id())
            .collect()
    };

    // Collect the starting positions based on the relevant options given.
    // If arguments were given, search for matches depending on whether or
    // not these should be treated as regular expressions. Otherwise, collect
    // the starting positions based on the direction we will be traversing.
    let args_given = !opts.functions().is_empty();
    let using_regexps = opts.using_regexps();
    let descending = opts.descending();
    let res = match (args_given, using_regexps, descending) {
        (true, true, _) => get_regexp_matches()?,
        (true, false, _) => get_exact_matches(),
        (false, _, true) => get_functions_default_desc(),
        (false, _, false) => get_functions_default(),
    };

    Ok(res)
}

/// Create a `PathsEntry` object for the given item.
fn create_entry(
    id: ir::Id,
    items: &ir::Items,
    opts: &opt::Paths,
    seen: &mut BTreeSet<ir::Id>,
) -> PathsEntry {
    // Determine the item's name and size.
    let item = &items[id];
    let name = item.name().to_string();
    let size = item.size();

    // Collect the `ir::Id` values of this entry's children, depending on
    // whether we are ascending or descending the IR-tree.
    let children_ids: Vec<ir::Id> = if opts.descending() {
        items
            .neighbors(id)
            .map(|id| id as ir::Id)
            .filter(|id| !seen.contains(id))
            .filter(|&id| id != items.meta_root())
            .collect()
    } else {
        items
            .predecessors(id)
            .map(|id| id as ir::Id)
            .filter(|id| !seen.contains(id))
            .filter(|&id| id != items.meta_root())
            .collect()
    };

    // Temporarily add the current item to the set of discovered nodes, and
    // create an entry for each child. Collect these into a `children` vector.
    seen.insert(id);
    let children = children_ids
        .into_iter()
        .map(|id| create_entry(id, items, opts, seen))
        .collect();
    seen.remove(&id);

    PathsEntry {
        name,
        size,
        children,
    }
}
