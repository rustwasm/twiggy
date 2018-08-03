//! Implementations of the analyses that `twiggy` runs on its IR.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate csv;
extern crate petgraph;
extern crate regex;
extern crate twiggy_ir as ir;
extern crate twiggy_opt as opt;
extern crate twiggy_traits as traits;

mod json;

use serde::ser::SerializeStruct;
use std::cmp;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::io;
use std::iter;

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
        writeln!(f)?;

        for (i, max_len) in maxs.iter().enumerate().take(self.header.len()) {
            if i == 0 {
                write!(f, "─")?;
            } else {
                write!(f, "─┼─")?;
            }
            for _ in 0..*max_len {
                write!(f, "─")?;
            }
        }
        writeln!(f)?;

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
            writeln!(f)?;
        }

        Ok(())
    }
}

struct Top {
    items: Vec<ir::Id>,
    opts: opt::Top,
}

impl traits::Emit for Top {
    #[cfg(feature = "emit_text")]
    fn emit_text(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        // A struct used to represent a row in the table that will be emitted.
        struct TableRow {
            size: u32,
            size_percent: f64,
            name: String,
        };

        // Helper function used to process an item, and return a struct
        // representing a row containing its size and name.
        fn process_item(id: ir::Id, items: &ir::Items, retained: bool) -> TableRow {
            let item = &items[id];
            let size = if retained {
                items.retained_size(id)
            } else {
                item.size()
            };
            let size_percent = (f64::from(size)) / (f64::from(items.size())) * 100.0;
            let name = item.name().to_string();
            TableRow {
                size,
                size_percent,
                name,
            }
        };

        // Helper function used to summnarize a sequence of table rows. This is
        // used to generate the remaining summary and total rows. Returns a tuple
        // containing the total size, total size percentage, and number of items.
        fn summarize_rows(rows: impl Iterator<Item = TableRow>) -> (u32, f64, u32) {
            rows.fold(
                (0, 0.0, 0),
                |(total_size, total_percent, remaining_count),
                 TableRow {
                     size, size_percent, ..
                 }| {
                    (
                        total_size + size,
                        total_percent + size_percent,
                        remaining_count + 1,
                    )
                },
            )
        }

        // Access the options that are relevant to emitting the correct output.
        let max_items = self.opts.max_items() as usize;
        let retained = self.opts.retained();
        let sort_label = if retained { "Retained" } else { "Shallow" };

        // Initialize a new table.
        let mut table = Table::with_header(vec![
            (Align::Right, format!("{} Bytes", sort_label)),
            (Align::Right, format!("{} %", sort_label)),
            (Align::Left, "Item".to_string()),
        ]);

        // Process the number of items specified, and add them to the table.
        self.items
            .iter()
            .take(max_items)
            .map(|&id| process_item(id, items, retained))
            .for_each(
                |TableRow {
                     size,
                     size_percent,
                     name,
                 }| {
                    table.add_row(vec![
                        size.to_string(),
                        format!("{:.2}%", size_percent),
                        name,
                    ])
                },
            );

        // Find the summary statistics by processing the remaining items.
        let remaining_rows = self
            .items
            .iter()
            .skip(max_items)
            .map(|&id| process_item(id, items, retained));
        let (rem_size, rem_size_percent, rem_count) = summarize_rows(remaining_rows);

        // If there were items remaining, add a summary row to the table.
        if rem_count > 0 {
            let rem_name_col = format!("... and {} more.", rem_count);
            let (rem_size_col, rem_size_percent_col) = if retained {
                ("...".to_string(), "...".to_string())
            } else {
                (rem_size.to_string(), format!("{:.2}%", rem_size_percent))
            };
            table.add_row(vec![rem_size_col, rem_size_percent_col, rem_name_col]);
        }

        // Add a row containing the totals to the table.
        let all_rows = self
            .items
            .iter()
            .map(|&id| process_item(id, items, retained));
        let (total_size, total_size_percent, total_count) = summarize_rows(all_rows);
        let total_name_col = format!("Σ [{} Total Rows]", total_count);
        let (total_size_col, total_size_percent_col) = if retained {
            ("...".to_string(), "...".to_string())
        } else {
            (
                total_size.to_string(),
                format!("{:.2}%", total_size_percent),
            )
        };
        table.add_row(vec![total_size_col, total_size_percent_col, total_name_col]);

        // Write the generated table out to the destination and return.
        write!(dest, "{}", &table)?;
        Ok(())
    }

    #[cfg(feature = "emit_json")]
    fn emit_json(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut arr = json::array(dest)?;

        let max_items = self.opts.max_items() as usize;
        let items_iter = self.items.iter();

        for &id in items_iter.take(max_items) {
            let item = &items[id];

            let mut obj = arr.object()?;
            obj.field("name", item.name())?;

            let size = item.size();
            let size_percent = f64::from(size) / f64::from(items.size()) * 100.0;
            obj.field("shallow_size", size)?;
            obj.field("shallow_size_percent", size_percent)?;

            if self.opts.retained() {
                let size = items.retained_size(id);
                let size_percent = f64::from(size) / f64::from(items.size()) * 100.0;
                obj.field("retained_size", size)?;
                obj.field("retained_size_percent", size_percent)?;
            }
        }

        Ok(())
    }

    #[cfg(feature = "emit_csv")]
    fn emit_csv(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut wtr = csv::Writer::from_writer(dest);

        #[derive(Serialize, Debug)]
        #[serde(rename_all = "PascalCase")]
        struct CsvRecord {
            name: String,
            shallow_size: u32,
            shallow_size_percent: f64,
            retained_size: Option<u32>,
            retained_size_percent: Option<f64>,
        }

        let max_items = self.opts.max_items() as usize;
        let items_iter = self.items.iter();

        for &id in items_iter.take(max_items) {
            let item = &items[id];

            let (shallow_size, shallow_size_percent) = {
                let size = item.size();
                let size_percent = f64::from(size) / f64::from(items.size()) * 100.0;
                (size, size_percent)
            };
            let (retained_size, retained_size_percent) = if self.opts.retained() {
                let size = items.retained_size(id);
                let size_percent = f64::from(size) / f64::from(items.size()) * 100.0;
                (Some(size), Some(size_percent))
            } else {
                (None, None)
            };

            wtr.serialize(CsvRecord {
                name: item.name().to_string(),
                shallow_size,
                shallow_size_percent,
                retained_size,
                retained_size_percent,
            })?;
            wtr.flush()?;
        }
        Ok(())
    }
}

/// Run the `top` analysis on the given IR items.
pub fn top(items: &mut ir::Items, opts: &opt::Top) -> Result<Box<traits::Emit>, traits::Error> {
    if opts.retaining_paths() {
        return Err(traits::Error::with_msg(
            "retaining paths are not yet implemented",
        ));
    }

    if opts.retained() {
        items.compute_retained_sizes();
    }

    let mut top_items: Vec<_> = items
        .iter()
        .filter(|item| item.id() != items.meta_root())
        .collect();

    top_items.sort_by(|a, b| {
        if opts.retained() {
            items
                .retained_size(b.id())
                .cmp(&items.retained_size(a.id()))
        } else {
            b.size().cmp(&a.size())
        }
    });

    let top_items: Vec<_> = top_items.into_iter().map(|i| i.id()).collect();

    let top = Top {
        items: top_items,
        opts: opts.clone(),
    };

    Ok(Box::new(top) as Box<traits::Emit>)
}

struct DominatorTree {
    tree: BTreeMap<ir::Id, Vec<ir::Id>>,
    items: Vec<ir::Id>,
    opts: opt::Dominators,
}

impl traits::Emit for DominatorTree {
    #[cfg(feature = "emit_text")]
    fn emit_text(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut table = Table::with_header(vec![
            (Align::Right, "Retained Bytes".to_string()),
            (Align::Right, "Retained %".to_string()),
            (Align::Left, "Dominator Tree".to_string()),
        ]);

        let opts = &self.opts;
        let mut row = 0 as u32;

        fn recursive_add_rows(
            table: &mut Table,
            items: &ir::Items,
            dominator_tree: &BTreeMap<ir::Id, Vec<ir::Id>>,
            depth: u32,
            mut row: &mut u32,
            opts: &opt::Dominators,
            id: ir::Id,
        ) {
            assert_eq!(id == items.meta_root(), depth == 0);

            if *row == opts.max_rows() {
                return;
            }

            if depth > opts.max_depth() {
                return;
            }

            if depth > 0 {
                let item = &items[id];

                let size = items.retained_size(id);
                let size_percent = (f64::from(size)) / (f64::from(items.size())) * 100.0;

                let mut label =
                    String::with_capacity(depth as usize * 4 + item.name().len() + "⤷ ".len());
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
                let mut children = children.to_vec();
                children.sort_by(|a, b| items.retained_size(*b).cmp(&items.retained_size(*a)));
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

        for id in &self.items {
            let start_depth = if *id == items.meta_root() { 0 } else { 1 };
            recursive_add_rows(
                &mut table,
                items,
                &self.tree,
                start_depth,
                &mut row,
                &opts,
                *id,
            );
        }

        write!(dest, "{}", &table)?;
        Ok(())
    }

    #[cfg(feature = "emit_json")]
    fn emit_json(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        fn recursive_add_children(
            items: &ir::Items,
            opts: &opt::Dominators,
            dominator_tree: &BTreeMap<ir::Id, Vec<ir::Id>>,
            id: ir::Id,
            obj: &mut json::Object,
        ) -> Result<(), traits::Error> {
            let item = &items[id];

            obj.field("name", item.name())?;

            let size = item.size();
            let size_percent = f64::from(size) / f64::from(items.size()) * 100.0;
            obj.field("shallow_size", size)?;
            obj.field("shallow_size_percent", size_percent)?;

            let size = items.retained_size(id);
            let size_percent = f64::from(size) / f64::from(items.size()) * 100.0;
            obj.field("retained_size", size)?;
            obj.field("retained_size_percent", size_percent)?;

            // TODO: this needs to do the filtering like how text formatting
            // does, but it would be nice to push that earlier, like `top` does.

            if let Some(children) = dominator_tree.get(&id) {
                let mut children = children.to_vec();
                children.sort_by(|a, b| items.retained_size(*b).cmp(&items.retained_size(*a)));

                let mut arr = obj.array("children")?;
                for child in children {
                    let mut obj = arr.object()?;
                    recursive_add_children(items, opts, dominator_tree, child, &mut obj)?;
                }
            }

            Ok(())
        }

        let mut obj = json::object(dest)?;
        for curr_id in &self.items {
            recursive_add_children(items, &self.opts, &self.tree, *curr_id, &mut obj)?;
        }

        Ok(())
    }

    #[cfg(feature = "emit_csv")]
    fn emit_csv(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut wtr = csv::Writer::from_writer(dest);
        fn recursive_add_children(
            items: &ir::Items,
            opts: &opt::Dominators,
            dominator_tree: &BTreeMap<ir::Id, Vec<ir::Id>>,
            id: ir::Id,
            wtr: &mut csv::Writer<&mut io::Write>,
        ) -> Result<(), traits::Error> {
            #[derive(Serialize, Debug)]
            #[serde(rename_all = "PascalCase")]
            struct CsvRecord {
                id: u64,
                name: String,
                shallow_size: u32,
                shallow_size_percent: f64,
                retained_size: u32,
                retained_size_percent: f64,
                immediate_dominator: u64,
            }

            let item = &items[id];
            let (size, size_percent) = (
                item.size(),
                f64::from(item.size()) / f64::from(items.size()) * 100.0,
            );
            let (retained_size, retained_size_percent) = (
                items.retained_size(id),
                f64::from(items.retained_size(id)) / f64::from(items.size()) * 100.0,
            );
            let idom = if let Some(idom) = items.immediate_dominators().get(&id) {
                idom.serializable()
            } else {
                id.serializable()
            };

            let rc = CsvRecord {
                id: item.id().serializable(),
                name: item.name().to_string(),
                shallow_size: size,
                shallow_size_percent: size_percent,
                retained_size,
                retained_size_percent,
                immediate_dominator: idom,
            };

            wtr.serialize(rc)?;
            wtr.flush()?;
            if let Some(children) = dominator_tree.get(&id) {
                let mut children = children.to_vec();
                children.sort_by(|a, b| items.retained_size(*b).cmp(&items.retained_size(*a)));
                for child in children {
                    recursive_add_children(items, opts, dominator_tree, child, wtr)?;
                }
            }
            Ok(())
        }

        recursive_add_children(items, &self.opts, &self.tree, items.meta_root(), &mut wtr)?;
        Ok(())
    }
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

    let tree = DominatorTree {
        tree: items.dominator_tree().clone(),
        items: dominator_items,
        opts: opts.clone(),
    };

    Ok(Box::new(tree) as Box<traits::Emit>)
}

struct Paths {
    items: Vec<ir::Id>,
    opts: opt::Paths,
}

impl traits::Emit for Paths {
    #[cfg(feature = "emit_text")]
    fn emit_text(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        fn recursive_callers(
            items: &ir::Items,
            seen: &mut BTreeSet<ir::Id>,
            table: &mut Table,
            depth: u32,
            mut paths: &mut u32,
            opts: &opt::Paths,
            id: ir::Id,
        ) {
            if opts.max_paths() == *paths || depth > opts.max_depth() {
                return;
            }

            if seen.contains(&id) || items.meta_root() == id {
                return;
            }

            let item = &items[id];

            let mut label = String::with_capacity(depth as usize * 4 + item.name().len());
            for _ in 1..depth {
                label.push_str("    ");
            }
            if depth > 0 {
                if opts.descending() {
                    label.push_str("  ↳ ");
                } else {
                    label.push_str("  ⬑ ");
                }
            }
            label.push_str(item.name());

            table.add_row(vec![
                if depth == 0 {
                    item.size().to_string()
                } else {
                    "".to_string()
                },
                if depth == 0 {
                    let size_percent = (f64::from(item.size())) / (f64::from(items.size())) * 100.0;
                    format!("{:.2}%", size_percent)
                } else {
                    "".to_string()
                },
                label,
            ]);

            seen.insert(id);

            if opts.descending() {
                for callee in items.neighbors(id) {
                    *paths += 1;
                    recursive_callers(items, seen, table, depth + 1, &mut paths, &opts, callee);
                }
            } else {
                for (i, caller) in items.predecessors(id).enumerate() {
                    if i > 0 {
                        *paths += 1;
                    }
                    recursive_callers(items, seen, table, depth + 1, &mut paths, &opts, caller);
                }
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
            let mut paths = 0 as u32;
            let mut seen = BTreeSet::new();
            recursive_callers(items, &mut seen, &mut table, 0, &mut paths, &opts, *id);
        }

        write!(dest, "{}", table)?;
        Ok(())
    }

    #[cfg(feature = "emit_json")]
    fn emit_json(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        fn recursive_callers(
            items: &ir::Items,
            seen: &mut BTreeSet<ir::Id>,
            obj: &mut json::Object,
            depth: u32,
            mut paths: &mut u32,
            opts: &opt::Paths,
            id: ir::Id,
        ) -> io::Result<()> {
            let item = &items[id];

            obj.field("name", item.name())?;

            let size = item.size();
            let size_percent = f64::from(size) / f64::from(items.size()) * 100.0;
            obj.field("shallow_size", size)?;
            obj.field("shallow_size_percent", size_percent)?;

            let mut callers = obj.array("callers")?;

            let depth = depth + 1;
            if depth <= opts.max_depth() {
                seen.insert(id);
                for (i, caller) in items.predecessors(id).enumerate() {
                    if seen.contains(&caller) || items.meta_root() == caller {
                        continue;
                    }

                    if i > 0 {
                        *paths += 1;
                    }
                    if opts.max_paths() == *paths {
                        break;
                    }

                    let mut obj = callers.object()?;
                    recursive_callers(items, seen, &mut obj, depth, &mut paths, &opts, caller)?;
                }
                seen.remove(&id);
            }

            Ok(())
        }

        let mut arr = json::array(dest)?;
        for id in &self.items {
            let mut paths = 0 as u32;
            let mut seen = BTreeSet::new();
            let mut obj = arr.object()?;
            recursive_callers(items, &mut seen, &mut obj, 0, &mut paths, &self.opts, *id)?;
        }

        Ok(())
    }

    #[cfg(feature = "emit_csv")]
    fn emit_csv(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut wtr = csv::Writer::from_writer(dest);
        fn recursive_callers(
            items: &ir::Items,
            seen: &mut BTreeSet<ir::Id>,
            depth: u32,
            mut paths: &mut u32,
            opts: &opt::Paths,
            id: ir::Id,
            wtr: &mut csv::Writer<&mut io::Write>,
        ) -> io::Result<()> {
            #[derive(Serialize, Debug)]
            #[serde(rename_all = "PascalCase")]
            struct CsvRecord {
                name: String,
                shallow_size: u32,
                shallow_size_percent: f64,
                path: Option<String>,
            }

            let item = &items[id];
            let size = item.size();
            let size_percent = f64::from(size) / f64::from(items.size()) * 100.0;
            let mut callers = items
                .predecessors(id)
                .into_iter()
                .map(|i| items[i].name())
                .collect::<Vec<&str>>();
            callers.push(item.name());
            let path = callers.join(" -> ");

            let record = CsvRecord {
                name: item.name().to_owned(),
                shallow_size: size,
                shallow_size_percent: size_percent,
                path: Some(path),
            };

            wtr.serialize(record)?;
            wtr.flush()?;

            let depth = depth + 1;
            if depth <= opts.max_depth() {
                seen.insert(id);
                for (i, caller) in items.predecessors(id).enumerate() {
                    if seen.contains(&caller) || items.meta_root() == caller {
                        continue;
                    }

                    if i > 0 {
                        *paths += 1;
                    }
                    if opts.max_paths() == *paths {
                        break;
                    }

                    recursive_callers(items, seen, depth, &mut paths, &opts, caller, wtr)?;
                }
                seen.remove(&id);
            }

            Ok(())
        }

        for id in &self.items {
            let mut paths = 0 as u32;
            let mut seen = BTreeSet::new();
            recursive_callers(items, &mut seen, 0, &mut paths, &self.opts, *id, &mut wtr)?;
        }

        Ok(())
    }
}

/// Find all retaining paths for the given items.
pub fn paths(items: &mut ir::Items, opts: &opt::Paths) -> Result<Box<traits::Emit>, traits::Error> {
    // The predecessor tree only needs to be computed if we are ascending
    // through the retaining paths.
    if !opts.descending() {
        items.compute_predecessors();
    }

    // This closure is used to initialize `functions` if no arguments are given
    // and we are ascending the retaining paths.
    let get_functions_default = || {
        let mut sorted_items: Vec<_> = items
            .iter()
            .filter(|item| item.id() != items.meta_root())
            .collect();
        sorted_items.sort_by(|a, b| b.size().cmp(&a.size()));
        sorted_items.iter().map(|item| item.id()).collect()
    };

    // This closure is used to initialize `functions` if no arguments are given
    // and we are descending the retaining paths.
    let get_functions_default_desc = || {
        let mut roots: Vec<_> = items
            .neighbors(items.meta_root())
            .map(|id| &items[id])
            .collect();
        roots.sort_by(|a, b| b.size().cmp(&a.size()));
        roots.into_iter().map(|item| item.id()).collect()
    };

    // Initialize the collection of Id values whose retaining paths we will emit.
    let functions: Vec<ir::Id> = if opts.functions().is_empty() {
        if opts.descending() {
            get_functions_default_desc()
        } else {
            get_functions_default()
        }
    } else if opts.using_regexps() {
        let regexps = regex::RegexSet::new(opts.functions())?;
        items
            .iter()
            .filter(|item| regexps.is_match(&item.name()))
            .map(|item| item.id())
            .collect()
    } else {
        opts.functions()
            .iter()
            .filter_map(|s| items.get_item_by_name(s))
            .map(|item| item.id())
            .collect()
    };

    let paths = Paths {
        items: functions,
        opts: opts.clone(),
    };

    Ok(Box::new(paths) as Box<traits::Emit>)
}

#[derive(Debug)]
struct Monos {
    monos: Vec<MonosEntry>,
}

#[derive(Debug, PartialEq, Eq)]
struct MonosEntry {
    name: String,
    insts: Vec<(String, u32)>,
    size: u32,
    bloat: u32,
}

impl PartialOrd for MonosEntry {
    fn partial_cmp(&self, rhs: &MonosEntry) -> Option<cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for MonosEntry {
    fn cmp(&self, rhs: &MonosEntry) -> cmp::Ordering {
        rhs.bloat
            .cmp(&self.bloat)
            .then(rhs.size.cmp(&self.size))
            .then(self.insts.cmp(&rhs.insts))
            .then(self.name.cmp(&rhs.name))
    }
}

impl traits::Emit for Monos {
    #[cfg(feature = "emit_text")]
    fn emit_text(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        struct TableRow {
            bloat: Option<u32>,
            bloat_percent: Option<f64>,
            size: u32,
            size_percent: f64,
            name: String,
        };

        // Given an entry representing a generic function and its various
        // monomorphizations, return a vector of table rows.
        fn process_entry<'a>(
            entry: &'a MonosEntry,
            total_size: f64,
        ) -> impl Iterator<Item = TableRow> + 'a {
            let MonosEntry {
                name,
                insts,
                size,
                bloat,
            } = entry;

            let get_size_percent = move |x: u32| f64::from(x) / total_size * 100.0;

            iter::once(TableRow {
                bloat: Some(*bloat),
                bloat_percent: Some(get_size_percent(*bloat)),
                size: *size,
                size_percent: get_size_percent(*size),
                name: name.to_string(),
            }).chain(insts.iter().map(move |(name, size)| TableRow {
                bloat: None,
                bloat_percent: None,
                size: *size,
                size_percent: get_size_percent(*size),
                name: format!("    {}", name),
            }))
        }

        let mut table = Table::with_header(vec![
            (Align::Right, "Apprx. Bloat Bytes".into()),
            (Align::Right, "Apprx. Bloat %".into()),
            (Align::Right, "Bytes".into()),
            (Align::Right, "%".into()),
            (Align::Left, "Monomorphizations".to_string()),
        ]);

        for TableRow {
            bloat,
            bloat_percent,
            size,
            size_percent,
            name,
        } in self
            .monos
            .iter()
            .flat_map(|mono| process_entry(mono, f64::from(items.size())))
        {
            table.add_row(vec![
                bloat.map(|b| b.to_string()).unwrap_or_default(),
                bloat_percent
                    .map(|b| format!("{:.2}%", b))
                    .unwrap_or_default(),
                size.to_string(),
                format!("{:.2}%", size_percent),
                name.clone(),
            ]);
        }
        write!(dest, "{}", &table)?;
        Ok(())
    }

    #[cfg(feature = "emit_json")]
    fn emit_json(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        // Given an entry representing a generic function and its various
        // monomorphizations, add its information to the given JSON object.
        fn process_entry(
            entry: &MonosEntry,
            obj: &mut json::Object,
            total_size: f64,
        ) -> Result<(), traits::Error> {
            let get_size_percent = |size: u32| (f64::from(size)) / total_size * 100.0;
            let MonosEntry {
                name,
                insts,
                size,
                bloat,
            } = entry;
            obj.field("generic", name.as_str())?;
            obj.field("approximate_monomorphization_bloat_bytes", *bloat)?;
            obj.field(
                "approximate_monomorphization_bloat_percent",
                get_size_percent(*bloat),
            )?;
            obj.field("total_size", *size)?;
            obj.field("total_size_percent", get_size_percent(*size))?;
            let mut monos = obj.array("monomorphizations")?;
            for (name, size, size_percent) in insts
                .iter()
                .map(|(name, size)| (name, size, get_size_percent(*size)))
            {
                let mut obj = monos.object()?;
                obj.field("name", name.as_str())?;
                obj.field("shallow_size", *size)?;
                obj.field("shallow_size_percent", size_percent)?;
            }
            Ok(())
        };

        let items_size = f64::from(items.size());
        let mut arr = json::array(dest)?;
        for entry in &self.monos {
            let mut obj = arr.object()?;
            process_entry(entry, &mut obj, items_size)?;
        }

        Ok(())
    }

    #[cfg(feature = "emit_csv")]
    fn emit_csv(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        // Calculate the total size of the collection of items, and define a
        // helper closure to calculate a percent value for a given u32 size.
        let items_size = f64::from(items.size());
        let get_size_percent = |size: u32| (f64::from(size)) / items_size * 100.0;

        #[derive(Debug, Default, Serialize)]
        #[serde(rename_all = "PascalCase")]
        struct Record {
            generic: Option<String>,
            approximate_monomorphization_bloat_bytes: Option<u32>,
            approximate_monomorphization_bloat_percent: Option<f64>,
            total_size: Option<u32>,
            total_size_percent: Option<f64>,
            monomorphizations: Option<String>,
        }

        // Given a single `MonosEntry` object, create a record object.
        let process_entry = |entry: &MonosEntry| -> Record {
            let monos = entry
                .insts
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>();
            Record {
                generic: Some(entry.name.clone()),
                approximate_monomorphization_bloat_bytes: Some(entry.bloat),
                approximate_monomorphization_bloat_percent: Some(get_size_percent(entry.bloat)),
                total_size: Some(entry.size),
                total_size_percent: Some(get_size_percent(entry.size)),
                monomorphizations: Some(monos.join(", ")),
            }
        };

        // Create a CSV writer and iterate through the monomorphization entries.
        // Process each record and pass it to the destination to be serialized.
        let mut wtr = csv::Writer::from_writer(dest);
        for entry in &self.monos {
            let record = process_entry(entry);
            wtr.serialize(record)?;
            wtr.flush()?;
        }
        Ok(())
    }
}

/// Find bloaty monomorphizations of generic functions.
pub fn monos(items: &mut ir::Items, opts: &opt::Monos) -> Result<Box<traits::Emit>, traits::Error> {
    // Type alias used to represent a map of generic function names and instantiations.
    type MonosMap<'a> = BTreeMap<&'a str, Vec<(String, u32)>>;

    fn collect_monomorphizations<'a>(items: &'a ir::Items) -> MonosMap {
        let unsorted_monos: BTreeMap<&'a str, BTreeSet<(String, u32)>> = items
            .iter()
            .filter_map(|item| {
                if let Some(generic) = item.monomorphization_of() {
                    Some((generic, item))
                } else {
                    None
                }
            }).fold(BTreeMap::new(), |mut monos, (generic, inst)| {
                monos
                    .entry(generic)
                    .or_insert_with(BTreeSet::new)
                    .insert((inst.name().to_string(), inst.size()));
                monos
            });

        unsorted_monos
            .into_iter()
            .map(|(generic, inst_set)| {
                let mut insts = inst_set.into_iter().collect::<Vec<_>>();
                insts.sort_by(|(a_name, a_size), (b_name, b_size)| {
                    b_size.cmp(a_size).then(a_name.cmp(b_name))
                });
                (generic, insts)
            }).collect()
    }

    // Helper function usedd to summarize a sequence of `MonosEntry` objects.
    // Returns a tuple representing the number of items summarized, the total
    // size of the items, and the total approximate potential savings.
    fn summarize_entries<'a>(entries: impl Iterator<Item = &'a MonosEntry>) -> (usize, u32, u32) {
        entries.fold(
            (0, 0, 0),
            |(total_cnt, total_size, total_savings),
             MonosEntry {
                 insts, size, bloat, ..
             }| {
                (
                    total_cnt + 1 + insts.len(),
                    total_size + size,
                    total_savings + bloat,
                )
            },
        )
    }

    // Helper function used to summarize a sequence of tuples representing
    // instantiations of a generic function. Returns a tuple representing the
    // number of instantiations found, and the total size.
    fn summarize_insts<'a>(entries: impl Iterator<Item = &'a (String, u32)>) -> (u32, u32) {
        entries.fold((0, 0), |(total_cnt, total_size), (_, size)| {
            (total_cnt + 1, total_size + size)
        })
    }

    // Find the approximate potential savings by calculating the benefits of
    // removing the largest instantiation, and the benefits of removing an
    // average instantiation. Returns a tuple containing total size, and bloat.
    fn calculate_total_and_bloat<'a>(insts: &[(String, u32)]) -> Option<(u32, u32)> {
        if let Some(max) = insts.iter().map(|(_, size)| size).max() {
            let total_size = insts.iter().map(|(_, size)| size).sum::<u32>();
            let inst_cnt = insts.len() as u32;
            let size_per_inst = total_size / inst_cnt;
            let avg_savings = size_per_inst * (inst_cnt - 1);
            let removing_largest_savings = total_size - max;
            let approx_potential_savings = cmp::min(avg_savings, removing_largest_savings);
            Some((total_size, approx_potential_savings))
        } else {
            None
        }
    }

    // Process all of the monorphizations, into a vector of `MonosEntry` objects.
    fn process_monomorphizations(monos_map: MonosMap, opts: &opt::Monos) -> Vec<MonosEntry> {
        let mut monos = monos_map
            .into_iter()
            .filter_map(|(g, insts)| {
                calculate_total_and_bloat(&insts).map(|(total, bloat)| (g, insts, total, bloat))
            }).map(|(g, mut insts, t, b)| {
                // Truncate `insts` according to the relevant options before
                // we map these values into `MonosEntry` objects.
                if opts.only_generics() {
                    insts.truncate(0);
                } else {
                    let max_monos = opts.max_monos() as usize;
                    let (rem_cnt, rem_size) = summarize_insts(insts.iter().skip(max_monos));
                    insts.truncate(max_monos);
                    if rem_cnt > 0 {
                        insts.push((format!("... and {} more.", rem_cnt), rem_size));
                    }
                };
                (g, insts, t, b)
            }).map(|(name, insts, size, bloat)| MonosEntry {
                name: name.to_string(),
                insts,
                size,
                bloat,
            }).collect::<Vec<_>>();
        monos.sort();
        monos
    }

    // Collect the options that will be needed.
    let max_generics = opts.max_generics() as usize;

    // Collect the monomorphizations of generic functions into a map, then
    // process the entries and sort the resulting vector.
    let monos_map = collect_monomorphizations(&items);
    let mut monos = process_monomorphizations(monos_map, &opts);

    // Create an entry to represent the remaining rows that will be truncated.
    let (rem_cnt, rem_size, rem_savings) = summarize_entries(monos.iter().skip(max_generics));
    let remaining = MonosEntry {
        name: format!("... and {} more.", rem_cnt),
        size: rem_size,
        insts: vec![],
        bloat: rem_savings,
    };

    // Create an entry to represent the 'total' summary.
    let (total_cnt, total_size, total_savings) = summarize_entries(monos.iter());
    let total = MonosEntry {
        name: format!("Σ [{} Total Rows]", total_cnt),
        size: total_size,
        insts: vec![],
        bloat: total_savings,
    };

    // Truncate the vector, and add the 'remaining' and 'total' summary entries.
    monos.truncate(max_generics);
    if rem_cnt > 0 {
        monos.push(remaining);
    }
    monos.push(total);
    Ok(Box::new(Monos { monos }) as Box<traits::Emit>)
}

#[derive(Debug)]
struct Diff {
    deltas: Vec<DiffEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiffEntry {
    name: String,
    delta: i64,
}

impl PartialOrd for DiffEntry {
    fn partial_cmp(&self, rhs: &DiffEntry) -> Option<cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for DiffEntry {
    fn cmp(&self, rhs: &DiffEntry) -> cmp::Ordering {
        rhs.delta
            .abs()
            .cmp(&self.delta.abs())
            .then(self.name.cmp(&rhs.name))
    }
}

impl serde::Serialize for DiffEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("DiffEntry", 2)?;
        state.serialize_field("DeltaBytes", &format!("{:+}", self.delta))?;
        state.serialize_field("Item", &self.name)?;
        state.end()
    }
}

impl traits::Emit for Diff {
    #[cfg(feature = "emit_text")]
    fn emit_text(
        &self,
        _items: &ir::Items,
        dest: &mut std::io::Write,
    ) -> Result<(), traits::Error> {
        let mut table = Table::with_header(vec![
            (Align::Right, "Delta Bytes".into()),
            (Align::Left, "Item".to_string()),
        ]);

        self.deltas
            .iter()
            .map(|entry| vec![format!("{:+}", entry.delta), entry.name.clone()])
            .for_each(|row| table.add_row(row));

        write!(dest, "{}", &table)?;
        Ok(())
    }

    #[cfg(feature = "emit_json")]
    fn emit_json(
        &self,
        _items: &ir::Items,
        dest: &mut std::io::Write,
    ) -> Result<(), traits::Error> {
        let mut arr = json::array(dest)?;

        for entry in &self.deltas {
            let mut obj = arr.object()?;
            obj.field("delta_bytes", entry.delta as f64)?;
            obj.field("name", entry.name.as_str())?;
        }

        Ok(())
    }

    #[cfg(feature = "emit_csv")]
    fn emit_csv(&self, _items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut wtr = csv::Writer::from_writer(dest);

        for entry in &self.deltas {
            wtr.serialize(entry)?;
            wtr.flush()?;
        }

        Ok(())
    }
}

/// Compute the diff between two sets of items.
pub fn diff(
    old_items: &mut ir::Items,
    new_items: &mut ir::Items,
    opts: &opt::Diff,
) -> Result<Box<traits::Emit>, traits::Error> {
    let max_items = opts.max_items() as usize;

    // Given a set of items, create a HashMap of the items' names and sizes.
    fn get_names_and_sizes(items: &ir::Items) -> HashMap<&str, i64> {
        items
            .iter()
            .map(|item| (item.name(), i64::from(item.size())))
            .collect()
    }

    // Collect the names and sizes of the items in the old and new collections.
    let old_sizes = get_names_and_sizes(old_items);
    let new_sizes = get_names_and_sizes(new_items);

    // Given an item name, create a `DiffEntry` object representing the
    // change in size, or an error if the name could not be found in
    // either of the item collections.
    let get_item_delta = |name: String| -> Result<DiffEntry, traits::Error> {
        let old_size = old_sizes.get::<str>(&name);
        let new_size = new_sizes.get::<str>(&name);
        let delta: i64 = match (old_size, new_size) {
            (Some(old_size), Some(new_size)) => new_size - old_size,
            (Some(old_size), None) => -old_size,
            (None, Some(new_size)) => *new_size,
            (None, None) => {
                return Err(traits::Error::with_msg(format!(
                    "Could not find item with name `{}`",
                    name
                )))
            }
        };
        Ok(DiffEntry { name, delta })
    };

    // Given a result returned by `get_item_delta`, return false if the result
    // represents an unchanged item. Ignore errors, these are handled separately.
    let unchanged_items_filter = |res: &Result<DiffEntry, traits::Error>| -> bool {
        if let Ok(DiffEntry { delta: 0, .. }) = res {
            false
        } else {
            true
        }
    };

    // Create a set of item names from the new and old item collections.
    let names = old_sizes
        .keys()
        .chain(new_sizes.keys())
        .map(|k| k.to_string())
        .collect::<HashSet<_>>();

    // Iterate through the set of item names, and use the closure above to map
    // each item into a `DiffEntry` object. Then, sort the collection.
    let mut deltas = names
        .into_iter()
        .map(get_item_delta)
        .filter(unchanged_items_filter)
        .collect::<Result<Vec<_>, traits::Error>>()?;
    deltas.sort();

    // Create an entry to summarize the diff rows that will be truncated.
    let (rem_cnt, rem_delta): (u32, i64) = deltas
        .iter()
        .skip(max_items)
        .fold((0, 0), |(cnt, rem_delta), DiffEntry { delta, .. }| {
            (cnt + 1, rem_delta + delta)
        });
    let remaining = DiffEntry {
        name: format!("... and {} more.", rem_cnt),
        delta: rem_delta,
    };

    // Create a `DiffEntry` representing the net change, and total row count.
    let total = DiffEntry {
        name: format!("Σ [{} Total Rows]", deltas.len()),
        delta: i64::from(new_items.size()) - i64::from(old_items.size()),
    };

    // Now that the 'remaining' and 'total' summary entries have been created,
    // truncate the vector of deltas before we box up the result, and push
    // the remaining and total rows to the deltas vector.
    deltas.truncate(max_items);
    deltas.push(remaining);
    deltas.push(total);

    // Return the results so that they can be emitted.
    let diff = Diff { deltas };
    Ok(Box::new(diff) as Box<traits::Emit>)
}

#[derive(Debug)]
struct Garbage {
    items: Vec<ir::Id>,
    limit: usize,
}

impl traits::Emit for Garbage {
    #[cfg(feature = "emit_text")]
    fn emit_text(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut table = Table::with_header(vec![
            (Align::Right, "Bytes".to_string()),
            (Align::Right, "Size %".to_string()),
            (Align::Left, "Garbage Item".to_string()),
        ]);

        for &id in self.items.iter().take(self.limit) {
            let item = &items[id];
            let size = item.size();
            let size_percent = (f64::from(size)) / (f64::from(items.size())) * 100.0;
            table.add_row(vec![
                size.to_string(),
                format!("{:.2}%", size_percent),
                item.name().to_string(),
            ]);
        }

        if self.items.len() > self.limit {
            table.add_row(vec![
                "...".to_string(),
                "...".to_string(),
                format!("... and {} more", self.items.len() - self.limit),
            ]);
        }

        let total_size: u32 = self.items.iter().map(|&id| items[id].size()).sum();
        let total_percent = (f64::from(total_size)) / (f64::from(items.size())) * 100.0;
        table.add_row(vec![
            total_size.to_string(),
            format!("{:.2}%", total_percent),
            "Σ".to_string(),
        ]);

        write!(dest, "{}", &table)?;
        Ok(())
    }

    #[cfg(feature = "emit_json")]
    fn emit_json(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut arr = json::array(dest)?;

        for &id in self.items.iter().take(self.limit) {
            let item = &items[id];

            let mut obj = arr.object()?;
            obj.field("name", item.name())?;

            let size = item.size();
            let size_percent = (f64::from(size)) / (f64::from(items.size())) * 100.0;
            obj.field("bytes", size)?;
            obj.field("size_percent", size_percent)?;
        }

        Ok(())
    }

    #[cfg(feature = "emit_csv")]
    fn emit_csv(&self, _items: &ir::Items, _dest: &mut io::Write) -> Result<(), traits::Error> {
        unimplemented!();
    }
}

/// Find items that are not transitively referenced by any exports or public functions.
pub fn garbage(items: &ir::Items, opts: &opt::Garbage) -> Result<Box<traits::Emit>, traits::Error> {
    fn get_reachable_items(items: &ir::Items) -> BTreeSet<ir::Id> {
        let mut reachable_items: BTreeSet<ir::Id> = BTreeSet::new();
        let mut dfs = petgraph::visit::Dfs::new(items, items.meta_root());
        while let Some(id) = dfs.next(&items) {
            reachable_items.insert(id);
        }
        reachable_items
    }

    let reachable_items = get_reachable_items(&items);
    let mut unreachable_items: Vec<_> = items
        .iter()
        .filter(|item| !reachable_items.contains(&item.id()))
        .collect();

    unreachable_items.sort_by(|a, b| b.size().cmp(&a.size()));

    let unreachable_items: Vec<_> = unreachable_items.iter().map(|item| item.id()).collect();

    let garbage_items = Garbage {
        items: unreachable_items,
        limit: opts.max_items() as usize,
    };

    Ok(Box::new(garbage_items) as Box<traits::Emit>)
}
