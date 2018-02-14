//! The `svelte` code size profiler.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate cpp_demangle;
extern crate frozen;
extern crate rustc_demangle;

use frozen::Frozen;
use std::cmp;
use std::collections::btree_map;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops;
use std::u32;

/// Build up a a set of `Items`.
#[derive(Debug)]
pub struct ItemsBuilder {
    id_counter: u32,
    size: u32,
    parsed: HashMap<*const (), Id>,
    items: BTreeMap<Id, Item>,
    edges: BTreeMap<Id, BTreeSet<Id>>,
    roots: BTreeSet<Id>,
}

impl ItemsBuilder {
    /// Construct a new builder, with the given size.
    pub fn new(size: u32) -> ItemsBuilder {
        ItemsBuilder {
            id_counter: 0,
            size,
            parsed: Default::default(),
            items: Default::default(),
            edges: Default::default(),
            roots: Default::default(),
        }
    }

    /// Add the given item to to the graph and return the `Id` that it was
    /// assigned.
    pub fn add_item<T>(&mut self, key: &T, mut item: Item) -> Id {
        let id = Id(self.id_counter);
        self.id_counter += 1;

        item.id = id;
        self.items.insert(id, item);

        let old_value = self.parsed.insert(key as *const T as *const (), id);
        assert!(
            old_value.is_none(),
            "should not parse the same key into multiple items"
        );

        id
    }

    /// Add the given item to the graph as a root and return the `Id` that it
    /// was assigned.
    pub fn add_root<T>(&mut self, key: &T, item: Item) -> Id {
        let id = self.add_item(key, item);
        self.roots.insert(id);
        id
    }

    /// Add an edge between the given keys that have already been parsed into
    /// items.
    pub fn add_edge<K, J>(&mut self, from: &K, to: &J) {
        let from_id = self.id_for_key(from);
        let to_id = self.id_for_key(to);
        self.edges
            .entry(from_id)
            .or_insert(BTreeSet::new())
            .insert(to_id);
    }

    /// Get the id for the item we parsed from the given key.
    pub fn id_for_key<T>(&self, key: &T) -> Id {
        let key = key as *const T as *const ();
        self.parsed[&key]
    }

    /// Finish building the IR graph and return the resulting `Items`.
    pub fn finish(self) -> Items {
        Items {
            size: self.size,
            items: Frozen::freeze(self.items),
            edges: Frozen::freeze(
                self.edges
                    .into_iter()
                    .map(|(from, tos)| (from, tos.into_iter().collect::<Vec<_>>()))
                    .collect(),
            ),
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
    size: u32,
    items: Frozen<BTreeMap<Id, Item>>,
    edges: Frozen<BTreeMap<Id, Vec<Id>>>,
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

    /// The size of the total binary, containing all items.
    pub fn size(&self) -> u32 {
        self.size
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
    demangled: Option<String>,
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
        let name = name.into();
        let demangled = demangle(&name);
        Item {
            id: Id(u32::MAX),
            name,
            demangled,
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
        if let Some(ref demangled) = self.demangled {
            demangled
        } else {
            &self.name
        }
    }
}

fn demangle(s: &str) -> Option<String> {
    if let Ok(sym) = rustc_demangle::try_demangle(s) {
        return Some(sym.to_string());
    }

    if let Ok(sym) = cpp_demangle::Symbol::new(s) {
        return Some(sym.to_string());
    }

    None
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
