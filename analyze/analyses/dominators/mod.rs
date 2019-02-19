use std::collections::BTreeMap;

use regex;

use twiggy_ir as ir;
use twiggy_opt as opt;
use twiggy_traits as traits;

use crate::analyses::utils;

mod emit;

struct DominatorTree {
    tree: BTreeMap<ir::Id, Vec<ir::Id>>,
    items: Vec<ir::Id>,
    unreachable_items: Vec<ir::Id>,
    opts: opt::Dominators,
}

/// Compute the dominator tree for the given IR graph.
pub fn dominators(
    items: &mut ir::Items,
    opts: &opt::Dominators,
) -> Result<Box<traits::Emit>, traits::Error> {
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

    let unreachable_items = if arguments.is_empty() {
        utils::get_unreachable_items(&items)
            .map(|item| item.id())
            .collect::<Vec<ir::Id>>()
    } else {
        Vec::new()
    };

    let tree = DominatorTree {
        tree: items.dominator_tree().clone(),
        items: dominator_items,
        unreachable_items,
        opts: opts.clone(),
    };

    Ok(Box::new(tree) as Box<traits::Emit>)
}
