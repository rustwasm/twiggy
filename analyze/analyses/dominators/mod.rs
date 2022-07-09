use std::collections::BTreeMap;

use regex;

use twiggy_ir as ir;
use twiggy_opt as opt;
use twiggy_traits as traits;

use crate::analyses::garbage;

mod emit;

struct DominatorTree {
    tree: BTreeMap<ir::Id, Vec<ir::Id>>,
    items: Vec<ir::Id>,
    opts: opt::Dominators,
    unreachable_items_summary: Option<UnreachableItemsSummary>,
}

struct UnreachableItemsSummary {
    count: usize,
    size: u32,
    size_percent: f64,
}

/// Compute the dominator tree for the given IR graph.
pub fn dominators(
    items: &mut ir::Items,
    opts: &opt::Dominators,
) -> anyhow::Result<Box<dyn traits::Emit>> {
    items.compute_dominator_tree();
    items.compute_dominators();
    items.compute_retained_sizes();
    items.compute_predecessors();

    let arguments = opts.items();
    let dominator_items = if arguments.is_empty() {
        vec![items.meta_root()]
    } else if opts.using_regexps() {
        let regexps = regex::RegexSet::new(arguments)?;
        let mut sorted_items: Vec<_> = items
            .iter()
            .filter(|item| regexps.is_match(&item.name()))
            .map(|item| item.id())
            .collect();
        sorted_items.sort_by_key(|id| -i64::from(items.retained_size(*id)));
        sorted_items
    } else {
        arguments
            .iter()
            .filter_map(|name| items.get_item_by_name(name))
            .map(|item| item.id())
            .collect()
    };

    let tree = DominatorTree {
        tree: items.dominator_tree().clone(),
        items: dominator_items,
        opts: opts.clone(),
        unreachable_items_summary: summarize_unreachable_items(items, opts),
    };

    Ok(Box::new(tree) as Box<_>)
}

fn summarize_unreachable_items(
    items: &mut ir::Items,
    opts: &opt::Dominators,
) -> Option<UnreachableItemsSummary> {
    let (size, count) = garbage::get_unreachable_items(&items)
        .map(|item| item.size())
        .fold((0, 0), |(s, c), curr| (s + curr, c + 1));
    if opts.items().is_empty() && size > 0 {
        Some(UnreachableItemsSummary {
            count,
            size,
            size_percent: (f64::from(size)) / (f64::from(items.size())) * 100.0,
        })
    } else {
        None
    }
}
