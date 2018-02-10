//! The `svelte` code size profiler.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

/// TODO FITZGEN
pub trait Extra {
    /// TODO FITZGEN
    type ItemsExtra;

    /// TODO FITZGEN
    type ItemExtra;
}

/// The architecture- and target-independent internal representation of
/// functions, sections, etc in a file that is being size profiled.
#[derive(Clone, Debug)]
pub struct Items<E> {
    items: BTreeSet<Item<E>>,
    extra: E::ItemsExtra,
}

/// TODO FITZGEN
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item<E: Extra> {
    kind: ItemKind<E>,
    extra: E::ItemExtra,
}

/// TODO FITZGEN
pub enum ItemKind<E: Extra> {
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
