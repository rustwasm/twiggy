//! Implementations of the analyses that `svelte` runs on its IR.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

#[macro_use]
extern crate failure;
extern crate svelte_ir as ir;
extern crate svelte_opt as opt;
extern crate svelte_traits as traits;

use failure::ResultExt;
use std::cmp;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, Copy)]
enum Align {
    Left,
    Right,
}

#[derive(Debug, Clone)]
struct Table {
    header: Vec<(Align, String)>,
    rows: Vec<Vec<String>>,
}

impl Table {
    fn with_header(header: Vec<(Align, String)>) -> Table {
        assert!(!header.is_empty());
        Table {
            header,
            rows: vec![],
        }
    }

    fn add_row(&mut self, row: Vec<String>) {
        assert_eq!(self.header.len(), row.len());
        self.rows.push(row);
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut maxs: Vec<_> = self.header.iter().map(|h| h.1.len()).collect();

        for row in &self.rows {
            for (i, x) in row.iter().enumerate() {
                maxs[i] = cmp::max(maxs[i], x.len());
            }
        }

        let last = self.header.len() - 1;

        for (i, h) in self.header.iter().map(|h| &h.1).enumerate() {
            if i == 0 {
                write!(f, " ")?;
            } else {
                write!(f, " │ ")?;
            }

            write!(f, "{}", h)?;
            if i != last {
                for _ in 0..maxs[i] - h.len() {
                    write!(f, " ")?;
                }
            }
        }
        write!(f, "\n")?;

        for i in 0..self.header.len() {
            if i == 0 {
                write!(f, "─")?;
            } else {
                write!(f, "─┼─")?;
            }
            for _ in 0..maxs[i] {
                write!(f, "─")?;
            }
        }
        write!(f, "\n")?;

        for row in &self.rows {
            for (i, (x, align)) in row.iter().zip(self.header.iter().map(|h| h.0)).enumerate() {
                if i == 0 {
                    write!(f, " ")?;
                } else {
                    write!(f, " ┊ ")?;
                }

                match align {
                    Align::Left => {
                        write!(f, "{}", x)?;
                        if i != last {
                            for _ in 0..maxs[i] - x.len() {
                                write!(f, " ")?;
                            }
                        }
                    }
                    Align::Right => {
                        for _ in 0..maxs[i] - x.len() {
                            write!(f, " ")?;
                        }
                        write!(f, "{}", x)?;
                    }
                }
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

struct Top {
    items: Vec<ir::Id>,
    opts: opt::Top,
}

impl traits::Emit for Top {
    fn emit_text(
        &self,
        items: &ir::Items,
        dest: &opt::OutputDestination,
    ) -> Result<(), failure::Error> {
        let mut dest = dest.open().context("could not open output destination")?;

        let sort_label = match self.opts.sort_by {
            opt::SortBy::Shallow => "Shallow",
            opt::SortBy::Retained => "Retained",
        };

        let mut table = Table::with_header(vec![
            (Align::Right, format!("{} Bytes", sort_label)),
            (Align::Right, format!("{} %", sort_label)),
            (Align::Left, "Item".to_string()),
        ]);

        for &id in &self.items {
            let item = &items[id];

            let size = match self.opts.sort_by {
                opt::SortBy::Shallow => item.size(),
                opt::SortBy::Retained => items.retained_size(id),
            };

            let size_percent = (size as f64) / (items.size() as f64) * 100.0;
            table.add_row(vec![
                size.to_string(),
                format!("{:.2}%", size_percent),
                item.name().to_string(),
            ]);
        }

        write!(&mut dest, "{}", &table)?;
        Ok(())
    }
}

/// Run the `top` analysis on the given IR items.
pub fn top(items: &mut ir::Items, opts: &opt::Top) -> Result<Box<traits::Emit>, failure::Error> {
    if opts.retaining_paths {
        bail!("retaining paths are not yet implemented");
    }

    if opts.sort_by == opt::SortBy::Retained {
        items.compute_retained_sizes();
    }

    let mut top_items: Vec<_> = items
        .iter()
        .filter(|item| item.id() != items.meta_root())
        .collect();

    top_items.sort_unstable_by(|a, b| match opts.sort_by {
        opt::SortBy::Shallow => b.size().cmp(&a.size()),
        opt::SortBy::Retained => items
            .retained_size(b.id())
            .cmp(&items.retained_size(a.id())),
    });

    if let Some(n) = opts.number {
        top_items.truncate(n as usize);
    }

    let top_items: Vec<_> = top_items.into_iter().map(|i| i.id()).collect();

    let top = Top {
        items: top_items,
        opts: opts.clone(),
    };

    Ok(Box::new(top) as Box<traits::Emit>)
}

struct DominatorTree {
    tree: BTreeMap<ir::Id, Vec<ir::Id>>,
    opts: opt::Dominators,
}

impl traits::Emit for DominatorTree {
    fn emit_text(
        &self,
        items: &ir::Items,
        dest: &opt::OutputDestination,
    ) -> Result<(), failure::Error> {
        let mut dest = dest.open().context("could not open output destination")?;

        let mut table = Table::with_header(vec![
            (Align::Right, "Retained Bytes".to_string()),
            (Align::Right, "Retained %".to_string()),
            (Align::Left, "Dominator Tree".to_string()),
        ]);

        let opts = &self.opts;

        let mut row = 0 as usize;

        fn recursive_add_rows(
            table: &mut Table,
            items: &ir::Items,
            dominator_tree: &BTreeMap<ir::Id, Vec<ir::Id>>,
            depth: usize,
            mut row: &mut usize,
            opts: &opt::Dominators,
            id: ir::Id,
        ) {
            assert_eq!(id == items.meta_root(), depth == 0);

            if let Some(max_rows) = opts.max_rows {
                if *row == max_rows {
                    return;
                }
            }

            if let Some(max_depth) = opts.max_depth {
                if depth > max_depth {
                    return;
                }
            }

            if depth > 0 {
                let item = &items[id];

                let size = items.retained_size(id);
                let size_percent = (size as f64) / (items.size() as f64) * 100.0;

                let mut label = String::with_capacity(depth * 4 + item.name().len() + "⤷ ".len());
                for _ in 2..depth {
                    label.push_str("    ");
                }
                if depth != 1 {
                    label.push_str("  ⤷ ");
                }
                label.push_str(item.name());

                table.add_row(vec![
                    size.to_string(),
                    format!("{:.2}%", size_percent),
                    label,
                ]);
            }

            if let Some(children) = dominator_tree.get(&id) {
                let mut children: Vec<_> = children.iter().cloned().collect();
                children
                    .sort_unstable_by(|a, b| items.retained_size(*b).cmp(&items.retained_size(*a)));
                for child in children {
                    *row += 1;
                    recursive_add_rows(
                        table,
                        items,
                        dominator_tree,
                        depth + 1,
                        &mut row,
                        &opts,
                        child,
                    );
                }
            }
        }

        recursive_add_rows(
            &mut table,
            items,
            &self.tree,
            0,
            &mut row,
            &opts,
            items.meta_root(),
        );
        write!(&mut dest, "{}", &table)?;
        Ok(())
    }
}

/// Compute the dominator tree for the given IR graph.
pub fn dominators(
    items: &mut ir::Items,
    opts: &opt::Dominators,
) -> Result<Box<traits::Emit>, failure::Error> {
    items.compute_dominator_tree();
    items.compute_retained_sizes();

    let tree = DominatorTree {
        tree: items.dominator_tree().clone(),
        opts: opts.clone(),
    };

    Ok(Box::new(tree) as Box<traits::Emit>)
}

struct Paths {
    items: Vec<ir::Id>,
    opts: opt::Paths,
}

impl traits::Emit for Paths {
    fn emit_text(
        &self,
        items: &ir::Items,
        dest: &opt::OutputDestination,
    ) -> Result<(), failure::Error> {
        fn recursive_callers(
            items: &ir::Items,
            seen: &mut BTreeSet<ir::Id>,
            table: &mut Table,
            depth: usize,
            mut paths: &mut usize,
            opts: &opt::Paths,
            id: ir::Id,
        ) {
            if opts.max_paths == *paths || depth > opts.max_depth {
                return;
            }

            if seen.contains(&id) || items.meta_root() == id {
                return;
            }

            let item = &items[id];

            let mut label = String::with_capacity(depth * 4 + item.name().len());
            for _ in 1..depth {
                label.push_str("    ");
            }
            if depth > 0 {
                label.push_str("  ⬑ ");
            }
            label.push_str(item.name());

            table.add_row(vec![
                if depth == 0 {
                    item.size().to_string()
                } else {
                    "".to_string()
                },
                if depth == 0 {
                    let size_percent = (item.size() as f64) / (items.size() as f64) * 100.0;
                    format!("{:.2}%", size_percent)
                } else {
                    "".to_string()
                },
                label,
            ]);

            seen.insert(id);
            for (i, caller) in items.predecessors(id).enumerate() {
                if i > 0 {
                    *paths += 1;
                }
                recursive_callers(items, seen, table, depth + 1, &mut paths, &opts, caller);
            }
            seen.remove(&id);
        }

        let mut table = Table::with_header(vec![
            (Align::Right, "Shallow Bytes".to_string()),
            (Align::Right, "Shallow %".to_string()),
            (Align::Left, "Retaining Paths".to_string()),
        ]);

        let opts = &self.opts;

        for id in &self.items {
            let mut paths = 0 as usize;
            let mut seen = BTreeSet::new();
            recursive_callers(items, &mut seen, &mut table, 0, &mut paths, &opts, *id);
        }

        let mut dest = dest.open().context("could not open output destination")?;
        write!(&mut dest, "{}", table)?;
        Ok(())
    }
}

/// Find all retaining paths for the given items.
pub fn paths(
    items: &mut ir::Items,
    opts: &opt::Paths,
) -> Result<Box<traits::Emit>, failure::Error> {
    items.compute_predecessors();

    let mut paths = Paths {
        items: Vec::with_capacity(opts.functions.len()),
        opts: opts.clone(),
    };

    let functions: BTreeSet<_> = opts.functions.iter().map(|s| s.as_str()).collect();

    for item in items.iter() {
        if functions.contains(item.name()) {
            paths.items.push(item.id());
        }
    }

    Ok(Box::new(paths) as Box<traits::Emit>)
}
