//! The `svelte` code size profiler.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate cpp_demangle;
extern crate frozen;
extern crate petgraph;
extern crate rustc_demangle;

mod graph_impl;

use frozen::Frozen;
use std::cmp;
use std::collections::btree_map;
use std::collections::{BTreeMap, BTreeSet};
use std::ops;
use std::slice;
use std::u32;

/// Build up a a set of `Items`.
#[derive(Debug)]
pub struct ItemsBuilder {
    size: u32,
    parsed: BTreeSet<Id>,
    items: BTreeMap<Id, Item>,
    data:  BTreeMap<u32, (Id, u32)>, // offset -> (id, len)
    edges: BTreeMap<Id, BTreeSet<Id>>,
    roots: BTreeSet<Id>,
}

impl ItemsBuilder {
    /// Construct a new builder, with the given size.
    pub fn new(size: u32) -> ItemsBuilder {
        ItemsBuilder {
            size,
            parsed: Default::default(),
            items: Default::default(),
            data:  Default::default(),
            edges: Default::default(),
            roots: Default::default(),
        }
    }

    /// Add the given item to to the graph and return the `Id` that it was
    /// assigned.
    pub fn add_item(&mut self, item: Item) -> Id {
        let id = item.id;
        self.items.insert(id, item);

        let old_value = self.parsed.insert(id);
        assert!(old_value, "should not parse the same key into multiple items");
        
        id
    }

    /// Add the given item to the graph as a root and return the `Id` that it
    /// was assigned.
    pub fn add_root(&mut self, item: Item) -> Id {
        let id = self.add_item(item);
        self.roots.insert(id);
        id
    }

    /// Add an edge between the given keys that have already been parsed into
    /// items.
    pub fn add_edge(&mut self, from: Id, to: Id) {
        self.edges
            .entry(from)
            .or_insert(BTreeSet::new())
            .insert(to);
    }

    /// Add a range of static data and the `Id` that defines it.
    pub fn link_data(&mut self, offset: i64, len: usize, id: Id) {
        if offset >= 0 && offset <= u32::MAX as i64 && offset as usize + len < u32::MAX as usize {
            self.data.insert(offset as u32, (id, len as u32));
        }
    }

    /// locate the data section defining memory at the given offset
    pub fn get_data(&self, offset: u32) -> Option<Id> {
        self.data.range(offset..).next().and_then(|(start, &(id, len))| {
            if offset < start + len {
                Some(id)
            } else {
                None
            }
        })                                         
    }

    /// Finish building the IR graph and return the resulting `Items`.
    pub fn finish(mut self) -> Items {
        let meta_root_id = Id(u32::MAX, u32::MAX);
        let meta_root = Item::new(meta_root_id, "<meta root>", 0, Misc::new());
        self.items.insert(meta_root_id, meta_root);
        self.edges.insert(meta_root_id, self.roots.clone());

        Items {
            size: self.size,
            dominator_tree: None,
            retained_sizes: None,
            predecessors: None,
            items: Frozen::freeze(self.items),
            edges: Frozen::freeze(
                self.edges
                    .into_iter()
                    .map(|(from, tos)| (from, tos.into_iter().collect::<Vec<_>>()))
                    .collect(),
            ),
            roots: Frozen::freeze(self.roots),
            meta_root: meta_root_id,
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
    dominator_tree: Option<BTreeMap<Id, Vec<Id>>>,
    retained_sizes: Option<BTreeMap<Id, u32>>,
    predecessors: Option<BTreeMap<Id, Vec<Id>>>,
    items: Frozen<BTreeMap<Id, Item>>,
    edges: Frozen<BTreeMap<Id, Vec<Id>>>,
    roots: Frozen<BTreeSet<Id>>,
    meta_root: Id,
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

    /// Iterate over an item's neighbors.
    pub fn neighbors(&self, id: Id) -> Neighbors {
        Neighbors {
            inner: self.edges
                .get(&id)
                .map_or_else(|| [].iter(), |edges| edges.iter()),
        }
    }

    /// Iterate over an item's neighbors.
    pub fn predecessors(&self, id: Id) -> Predecessors {
        Predecessors {
            inner: self.predecessors
                .as_ref()
                .expect("To access predecessors, must have already called compute_predecessors")
                .get(&id)
                .map_or_else(|| [].iter(), |edges| edges.iter()),
        }
    }

    /// The size of the total binary, containing all items.
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Get the id of the "meta root" which is a single root item with edges to
    /// all of the real roots.
    pub fn meta_root(&self) -> Id {
        self.meta_root
    }

    /// Force computation of predecessors.
    pub fn compute_predecessors(&mut self) {
        if self.predecessors.is_some() {
            return;
        }

        let mut predecessors = BTreeMap::new();

        for (from, tos) in self.edges.iter() {
            for to in tos {
                predecessors
                    .entry(*to)
                    .or_insert_with(|| BTreeSet::new())
                    .insert(*from);
            }
        }

        self.predecessors = Some(
            predecessors
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().collect()))
                .collect(),
        );
    }

    /// Force computation of the dominator tree.
    pub fn compute_dominator_tree(&mut self) {
        if self.dominator_tree.is_some() {
            return;
        }

        let mut dominator_tree = BTreeMap::new();
        let dominators = petgraph::algo::dominators::simple_fast(&*self, self.meta_root);
        for item in self.iter() {
            if let Some(idom) = dominators.immediate_dominator(item.id()) {
                dominator_tree
                    .entry(idom)
                    .or_insert(BTreeSet::new())
                    .insert(item.id());
            }
        }

        self.dominator_tree = Some(
            dominator_tree
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().collect()))
                .collect(),
        );
    }

    /// Get a reference to the dominator tree.
    ///
    /// Must have already called `compute_dominator_tree`.
    pub fn dominator_tree(&self) -> &BTreeMap<Id, Vec<Id>> {
        self.dominator_tree
            .as_ref()
            .expect("must call compute_dominator_tree before calling dominator_tree")
    }

    /// Force computation of the retained sizes of each IR item.
    pub fn compute_retained_sizes(&mut self) {
        if self.retained_sizes.is_some() {
            return;
        }
        self.compute_dominator_tree();

        fn recursive_retained_size(
            retained_sizes: &mut BTreeMap<Id, u32>,
            items: &Items,
            item: &Item,
            dominator_tree: &BTreeMap<Id, Vec<Id>>,
        ) -> u32 {
            // Although the dominator tree cannot have cycles, because we
            // compute retained sizes in item iteration order, rather than from
            // the bottom of the dominator tree up, it is possible we have
            // already computed the retained sizes for subtrees.
            if let Some(rsize) = retained_sizes.get(&item.id()) {
                return *rsize;
            }

            let mut rsize = item.size();
            if let Some(children) = dominator_tree.get(&item.id()) {
                for child in children {
                    rsize += recursive_retained_size(
                        retained_sizes,
                        items,
                        &items[*child],
                        dominator_tree,
                    );
                }
            }

            let old_value = retained_sizes.insert(item.id(), rsize);
            // The dominator tree is a proper tree, so there shouldn't be
            // any cycles.
            assert!(old_value.is_none());
            rsize
        }

        let mut retained_sizes = BTreeMap::new();
        {
            let dominator_tree = self.dominator_tree.as_ref().unwrap();
            for item in self.iter() {
                recursive_retained_size(&mut retained_sizes, self, item, dominator_tree);
            }
        }
        self.retained_sizes = Some(retained_sizes);
    }

    /// Get the given item's retained size.
    pub fn retained_size(&self, id: Id) -> u32 {
        self.retained_sizes
            .as_ref()
            .expect(
                "Cannot call retained_sizes unless compute_retained_sizes \
                 has already been called",
            )
            .get(&id)
            .cloned()
            .unwrap()
    }
}

/// An iterator over an item's neighbors.
#[derive(Debug)]
pub struct Neighbors<'a> {
    inner: slice::Iter<'a, Id>,
}

impl<'a> Iterator for Neighbors<'a> {
    type Item = Id;

    #[inline]
    fn next(&mut self) -> Option<Id> {
        self.inner.next().cloned()
    }
}

/// An iterator over an item's predecessors.
#[derive(Debug)]
pub struct Predecessors<'a> {
    inner: slice::Iter<'a, Id>,
}

impl<'a> Iterator for Predecessors<'a> {
    type Item = Id;

    #[inline]
    fn next(&mut self) -> Option<Id> {
        self.inner.next().cloned()
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u32, u32);
impl Id {
    /// create `Id` for a the given section
    pub fn section(section: usize) -> Id {
        assert!(section < u32::MAX as usize);
        Id(section as u32, u32::MAX)
    }
    /// create `Id` for a given entry if a given section
    pub fn entry(section: usize, index: usize) -> Id {
        assert!(section < u32::MAX as usize);
        assert!(index < u32::MAX as usize);
        Id(section as u32, index as u32)
    }
    /// the root index
    pub fn root() -> Id {
        Id(u32::MAX, u32::MAX)
    }
}
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
    pub fn new<S, K>(id: Id, name: S, size: u32, kind: K) -> Item
    where
        S: Into<String>,
        K: Into<ItemKind>,
    {
        let name = name.into();
        let demangled = demangle(&name);
        Item {
            id: id,
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
    ty: Option<String>
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
