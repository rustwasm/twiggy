//! The `svelte` code size profiler.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate frozen;

use frozen::Frozen;
use std::cmp;
use std::collections::BTreeSet;

/// Build up a a set of `ir::Items`.
#[derive(Debug)]
pub struct ItemsBuilder {
    id_counter: u32,
    items: BTreeSet<Item>,
}

/// The architecture- and target-independent internal representation of
/// functions, sections, etc in a file that is being size profiled.
#[derive(Debug)]
pub struct Items {
    items: Frozen<BTreeSet<Item>>,
}

/// An item's unique identifier.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(u32);

/// An item in the binary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    id: Id,
    kind: Kind,
}

impl PartialOrd for Item {
    fn partial_cmp(&self, rhs: &Item) -> Option<cmp::Ordering> {
        self.id.partial_cmp(&rhs.id)
    }
}

impl Ord for Item {
    fn cmp(&self, rhs: &Item) -> cmp::Ordering {
        self.id.cmp(&rhs.id)
    }
}

/// The kind of item in the binary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
