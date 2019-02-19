use std::collections::BTreeSet;

use petgraph::visit::Walker;

use twiggy_ir as ir;

pub fn get_unreachable_items<'a>(items: &'a ir::Items) -> impl Iterator<Item = &'a ir::Item> {
    let reachable_items = petgraph::visit::Dfs::new(items, items.meta_root())
        .iter(&items)
        .collect::<BTreeSet<ir::Id>>();
    items
        .iter()
        .filter(move |item| !reachable_items.contains(&item.id()))
}
