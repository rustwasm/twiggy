use super::{Id, Items, Neighbors};
use petgraph::visit;
use std::collections::HashSet;

impl visit::GraphBase for Items {
    type EdgeId = ();
    type NodeId = Id;
}

impl visit::Visitable for Items {
    type Map = HashSet<Id>;

    #[inline]
    fn visit_map(&self) -> Self::Map {
        HashSet::with_capacity(self.items.len())
    }

    #[inline]
    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
    }
}

impl<'a> visit::IntoNeighbors for &'a Items {
    type Neighbors = Neighbors<'a>;

    #[inline]
    fn neighbors(self, id: Id) -> Self::Neighbors {
        self.neighbors(id)
    }
}

impl visit::NodeCount for Items {
    #[inline]
    fn node_count(&self) -> usize {
        self.items.len()
    }
}
