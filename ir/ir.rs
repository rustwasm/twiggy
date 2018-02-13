//! The `svelte` code size profiler.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate frozen;

use frozen::Frozen;
use std::cmp;
use std::collections::btree_map;
use std::collections::{BTreeMap, BTreeSet};
use std::ops;
use std::u32;

/// Build up a a set of `Items`.
#[derive(Debug)]
pub struct ItemsBuilder {
    id_counter: u32,
    items: BTreeMap<Id, Item>,
    roots: BTreeSet<Id>,
}

impl Default for ItemsBuilder {
    fn default() -> ItemsBuilder {
        ItemsBuilder {
            id_counter: 0,
            items: Default::default(),
            roots: Default::default(),
        }
    }
}

impl ItemsBuilder {
    /// Add the given item to to the graph and return the `Id` that it was
    /// assigned.
    pub fn add_item(&mut self, mut item: Item) -> Id {
        let id = Id(self.id_counter);
        self.id_counter += 1;

        item.id = id;
        self.items.insert(id, item);

        id
    }

    /// Add the given item to the graph as a root and return the `Id` that it
    /// was assigned.
    pub fn add_root(&mut self, item: Item) -> Id {
        let id = self.add_item(item);
        self.roots.insert(id);
        id
    }

    /// Finish building the IR graph and return the resulting `Items`.
    pub fn finish(self) -> Items {
        Items {
            items: Frozen::freeze(self.items),
            roots: Frozen::freeze(self.roots),
        }
    }
}

/// The architecture- and target-independent internal representation of
/// functions, sections, etc in a file that is being size profiled.
///
/// Constructed with `ItemsBuilder`.
#[derive(Debug)]
pub struct Items {
    items: Frozen<BTreeMap<Id, Item>>,
    roots: Frozen<BTreeSet<Id>>,
}

impl ops::Index<Id> for Items {
    type Output = Item;

    fn index(&self, id: Id) -> &Item {
        &self.items[&id]
    }
}

impl Items {
    /// Iterate over all of the IR items.
    pub fn iter(&self) -> Iter {
        Iter {
            inner: self.items.iter(),
        }
    }
}

/// An iterator over IR items. Created by `Items::iter`.
#[derive(Clone, Debug)]
pub struct Iter<'a> {
    inner: btree_map::Iter<'a, Id, Item>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, item)| item)
    }
}

/// An item's unique identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(u32);

/// An item in the binary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    id: Id,
    name: String,
    size: u32,
    kind: ItemKind,
}

impl Item {
    /// Construct a new `Item` of the given kind.
    pub fn new<S, K>(name: S, size: u32, kind: K) -> Item
    where
        S: Into<String>,
        K: Into<ItemKind>,
    {
        Item {
            id: Id(u32::MAX),
            name: name.into(),
            size,
            kind: kind.into(),
        }
    }

    /// Get this item's identifier.
    #[inline]
    pub fn id(&self) -> Id {
        self.id
    }

    /// Get this item's size.
    #[inline]
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Get this item's name.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
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
pub enum ItemKind {
    /// Executable code. Function bodies.
    Code(Code),

    /// Data inside the binary that may or may not end up loaded into memory
    /// with the executable code.
    Data(Data),

    /// Debugging symbols and information, such as a DWARF section.
    Debug(DebugInfo),

    /// Miscellaneous item. Perhaps metadata. Perhaps something else.
    Misc(Misc),
}

impl From<Code> for ItemKind {
    fn from(c: Code) -> ItemKind {
        ItemKind::Code(c)
    }
}

impl From<Data> for ItemKind {
    fn from(d: Data) -> ItemKind {
        ItemKind::Data(d)
    }
}

impl From<DebugInfo> for ItemKind {
    fn from(d: DebugInfo) -> ItemKind {
        ItemKind::Debug(d)
    }
}

impl From<Misc> for ItemKind {
    fn from(m: Misc) -> ItemKind {
        ItemKind::Misc(m)
    }
}

/// Executable code. Function bodies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Code;

impl Code {
    /// Construct a new IR item for executable code.
    pub fn new() -> Code {
        Code
    }
}

/// Data inside the binary that may or may not end up loaded into memory
/// with the executable code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Data {
    ty: Option<String>,
}

impl Data {
    /// Construct a new `Data` that has a type of the given type name, if known.
    pub fn new(ty: Option<String>) -> Data {
        Data { ty }
    }
}

/// Debugging symbols and information, such as DWARF sections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugInfo;

impl DebugInfo {
    /// Construct a new IR item for debug information and symbols.
    pub fn new() -> DebugInfo {
        DebugInfo
    }
}

/// Miscellaneous item. Perhaps metadata. Perhaps something else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Misc;

impl Misc {
    /// Construct a new miscellaneous IR item.
    pub fn new() -> Misc {
        Misc
    }
}
