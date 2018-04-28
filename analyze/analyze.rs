//! Implementations of the analyses that `twiggy` runs on its IR.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate csv as csv_sys;
#[macro_use]
extern crate serde_derive;
extern crate twiggy_ir as ir;
extern crate twiggy_opt as opt;
extern crate twiggy_traits as traits;

mod csv;
mod json;

use std::cmp;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io;

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
    fn emit_text(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let sort_label = if self.opts.retained() {
            "Retained"
        } else {
            "Shallow"
        };

        let mut table = Table::with_header(vec![
            (Align::Right, format!("{} Bytes", sort_label)),
            (Align::Right, format!("{} %", sort_label)),
            (Align::Left, "Item".to_string()),
        ]);

        for &id in &self.items {
            let item = &items[id];

            let size = if self.opts.retained() {
                items.retained_size(id)
            } else {
                item.size()
            };

            let size_percent = (f64::from(size)) / (f64::from(items.size())) * 100.0;
            table.add_row(vec![
                size.to_string(),
                format!("{:.2}%", size_percent),
                item.name().to_string(),
            ]);
        }

        write!(dest, "{}", &table)?;
        Ok(())
    }

    fn emit_json(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut arr = json::array(dest)?;

        for &id in &self.items {
            let item = &items[id];

            let mut obj = arr.object()?;
            obj.field("name", item.name())?;

            let size = item.size();
            let size_percent = (size as f64) / (items.size() as f64) * 100.0;
            obj.field("shallow_size", size)?;
            obj.field("shallow_size_percent", size_percent)?;

            if self.opts.retained() {
                let size = items.retained_size(id);
                let size_percent = (size as f64) / (items.size() as f64) * 100.0;
                obj.field("retained_size", size)?;
                obj.field("retained_size_percent", size_percent)?;
            }
        }

        Ok(())
    }

    fn emit_csv(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut wtr = csv_sys::Writer::from_writer(dest);

        for &id in &self.items {
            let item = &items[id];

            let (shallow_size, shallow_size_percent) = {
                let size = item.size();
                let size_percent = (size as f64) / (items.size() as f64) * 100.0;
                (size, size_percent)
            };
            let (retained_size, retained_size_percent) = if self.opts.retained {
                let size = items.retained_size(id);
                let size_percent = (size as f64) / (items.size() as f64) * 100.0;
                (Some(size), Some(size_percent))
            } else {
                (None, None)
            };

            wtr.serialize(csv::CsvRecord {
                name: item.name().to_string(),
                shallow_size: shallow_size,
                shallow_size_percent: shallow_size_percent,
                retained_size: retained_size,
                retained_size_percent: retained_size_percent,
                ..Default::default()
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

    top_items.sort_by(|a, b| match opts.retained() {
        false => b.size().cmp(&a.size()),
        true => items
            .retained_size(b.id())
            .cmp(&items.retained_size(a.id())),
    });

    top_items.truncate(opts.number() as usize);

    let top_items: Vec<_> = top_items.into_iter().map(|i| i.id()).collect();

    let top = Top {
        items: top_items,
        opts: opts.clone(),
    };

    Ok(Box::new(top) as Box<traits::Emit>)
}

struct DominatorTree {
    tree: BTreeMap<ir::Id, Vec<ir::Id>>,
    root_id: ir::Id,
    opts: opt::Dominators,
}

impl traits::Emit for DominatorTree {
    fn emit_text(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut table = Table::with_header(vec![
            (Align::Right, "Retained Bytes".to_string()),
            (Align::Right, "Retained %".to_string()),
            (Align::Left, "Dominator Tree".to_string()),
        ]);

        let opts = &self.opts;
        let mut row = 0 as u32;
        let start_depth = if self.root_id == items.meta_root() {
            0
        } else {
            1
        };

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
                let mut children: Vec<_> = children.iter().cloned().collect();
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

        recursive_add_rows(
            &mut table,
            items,
            &self.tree,
            start_depth,
            &mut row,
            &opts,
            self.root_id,
        );
        write!(dest, "{}", &table)?;
        Ok(())
    }

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
            let size_percent = (size as f64) / (items.size() as f64) * 100.0;
            obj.field("shallow_size", size)?;
            obj.field("shallow_size_percent", size_percent)?;

            let size = items.retained_size(id);
            let size_percent = (size as f64) / (items.size() as f64) * 100.0;
            obj.field("retained_size", size)?;
            obj.field("retained_size_percent", size_percent)?;

            // TODO: this needs to do the filtering like how text formatting
            // does, but it would be nice to push that earlier, like `top` does.

            if let Some(children) = dominator_tree.get(&id) {
                let mut children: Vec<_> = children.iter().cloned().collect();
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
        recursive_add_children(items, &self.opts, &self.tree, self.root_id, &mut obj)
    }

    fn emit_csv(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut wtr = csv_sys::Writer::from_writer(dest);
        fn recursive_add_children<'a>(
            items: &ir::Items,
            opts: &opt::Dominators,
            dominator_tree: &BTreeMap<ir::Id, Vec<ir::Id>>,
            id: ir::Id,
            wtr: &mut csv_sys::Writer<&mut io::Write>,
        ) -> Result<(), traits::Error> {
            let item = &items[id];
            let (size, size_percent) = (
                item.size(),
                (item.size() as f64) / (items.size() as f64) * 100.0,
            );
            let (retained_size, retained_size_percent) = (
                items.retained_size(id),
                (items.retained_size(id) as f64) / (items.size() as f64) * 100.0,
            );

            let rc = csv::CsvRecord {
                id: Some(item.id().0),
                name: item.name().to_string(),
                shallow_size: size,
                shallow_size_percent: size_percent,
                retained_size: Some(retained_size),
                retained_size_percent: Some(retained_size_percent),
                // TODO CSMOE: find immediate dominator
                immediate_dominator: Some(id.0),
                ..Default::default()
            };

            wtr.serialize(rc)?;
            wtr.flush()?;
            if let Some(children) = dominator_tree.get(&id) {
                let mut children: Vec<_> = children.iter().cloned().collect();
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
    items.compute_retained_sizes();

    let subtree = opts.subtree();
    let root_id = match subtree.is_empty() {
        true => items.meta_root(),
        false => items
            .get_item_by_name(&subtree)
            .map(|item| item.id())
            .unwrap_or(items.meta_root()),
    };

    let tree = DominatorTree {
        tree: items.dominator_tree().clone(),
        root_id,
        opts: opts.clone(),
    };

    Ok(Box::new(tree) as Box<traits::Emit>)
}

struct Paths {
    items: Vec<ir::Id>,
    opts: opt::Paths,
}

impl traits::Emit for Paths {
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
            let size_percent = (size as f64) / (items.size() as f64) * 100.0;
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

    fn emit_csv(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut wtr = csv_sys::Writer::from_writer(dest);
        fn recursive_callers(
            items: &ir::Items,
            seen: &mut BTreeSet<ir::Id>,
            depth: u32,
            mut paths: &mut u32,
            opts: &opt::Paths,
            id: ir::Id,
            wtr: &mut csv_sys::Writer<&mut io::Write>,
        ) -> io::Result<()> {
            let item = &items[id];
            let size = item.size();
            let size_percent = (size as f64) / (items.size() as f64) * 100.0;
            let mut path = String::with_capacity(item.name().len());
            path.push_str(item.name());

            let record = csv::CsvRecord {
                name: item.name().to_owned(),
                shallow_size: size,
                shallow_size_percent: size_percent,
                path: Some(path),
                ..Default::default()
            };

            wtr.serialize(record)?;
            wtr.flush()?;

            let depth = depth + 1;
            if depth <= opts.max_depth {
                seen.insert(id);
                for (i, caller) in items.predecessors(id).enumerate() {
                    if seen.contains(&caller) || items.meta_root() == caller {
                        continue;
                    }

                    if i > 0 {
                        *paths += 1;
                    }
                    if opts.max_paths == *paths {
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
    if !opts.descending() {
        items.compute_predecessors();
    }

    let functions: Vec<ir::Id> = match opts.functions().is_empty() {
        true => {
            if opts.descending() {
                let mut roots: Vec<_> = items
                    .neighbors(items.meta_root())
                    .map(|id| &items[id])
                    .collect();
                roots.sort_by(|a, b| b.size().cmp(&a.size()));
                roots.into_iter().map(|item| item.id()).collect()
            } else {
                let mut sorted_items: Vec<_> = items
                    .iter()
                    .filter(|item| item.id() != items.meta_root())
                    .collect();
                sorted_items.sort_by(|a, b| b.size().cmp(&a.size()));
                sorted_items.iter().map(|item| item.id()).collect()
            }
        }
        false => opts.functions()
            .iter()
            .filter_map(|s| items.get_item_by_name(s))
            .map(|item| item.id())
            .collect(),
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
    generic: String,
    insts: Vec<ir::Id>,
    total: u32,
    approx_potential_savings: u32,
}

impl PartialOrd for MonosEntry {
    fn partial_cmp(&self, rhs: &MonosEntry) -> Option<cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for MonosEntry {
    fn cmp(&self, rhs: &MonosEntry) -> cmp::Ordering {
        rhs.approx_potential_savings
            .cmp(&self.approx_potential_savings)
            .then(self.insts.cmp(&rhs.insts))
            .then(self.generic.cmp(&rhs.generic))
    }
}

impl traits::Emit for Monos {
    fn emit_text(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut table = Table::with_header(vec![
            (Align::Right, "Apprx. Bloat Bytes".into()),
            (Align::Right, "Apprx. Bloat %".into()),
            (Align::Right, "Bytes".into()),
            (Align::Right, "%".into()),
            (Align::Left, "Monomorphizations".to_string()),
        ]);

        for entry in &self.monos {
            let total_percent = (f64::from(entry.total)) / (f64::from(items.size())) * 100.0;
            let approx_potential_savings_percent =
                (f64::from(entry.approx_potential_savings)) / (f64::from(items.size())) * 100.0;
            table.add_row(vec![
                entry.approx_potential_savings.to_string(),
                format!("{:.2}%", approx_potential_savings_percent),
                entry.total.to_string(),
                format!("{:.2}%", total_percent),
                entry.generic.clone(),
            ]);

            for &id in &entry.insts {
                let item = &items[id];

                let size = item.size();
                let size_percent = (f64::from(size)) / (f64::from(items.size())) * 100.0;

                table.add_row(vec![
                    "".into(),
                    "".into(),
                    size.to_string(),
                    format!("{:.2}%", size_percent),
                    format!("    {}", item.name()),
                ]);
            }
        }

        write!(dest, "{}", &table)?;
        Ok(())
    }

    fn emit_json(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut arr = json::array(dest)?;

        for entry in &self.monos {
            let mut obj = arr.object()?;
            obj.field("generic", &entry.generic[..])?;

            obj.field(
                "approximate_monomorphization_bloat_bytes",
                entry.approx_potential_savings,
            )?;
            let approx_potential_savings_percent =
                (f64::from(entry.approx_potential_savings)) / (f64::from(items.size())) * 100.0;
            obj.field(
                "approximate_monomorphization_bloat_percent",
                approx_potential_savings_percent,
            )?;

            obj.field("total_size", entry.total)?;
            let total_percent = (f64::from(entry.total)) / (f64::from(items.size())) * 100.0;
            obj.field("total_size_percent", total_percent)?;

            let mut monos = obj.array("monomorphizations")?;
            for &id in &entry.insts {
                let item = &items[id];

                let mut obj = monos.object()?;
                obj.field("name", item.name())?;

                let size = item.size();
                obj.field("shallow_size", size)?;

                let size_percent = (f64::from(size)) / (f64::from(items.size())) * 100.0;
                obj.field("shallow_size_percent", size_percent)?;
            }
        }

        Ok(())
    }

    fn emit_csv(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        #[derive(Debug, Default, Serialize)]
        struct Record {
            generic: Option<String>,
            approximate_monomorphization_bloat_bytes: Option<u32>,
            approximate_monomorphization_bloat_percent: Option<f64>,
            total_size: Option<u32>,
            total_size_percent: Option<f64>,
            monomorphizations: csv::CsvRecord,
        }

        let mut wtr = csv_sys::Writer::from_writer(dest);

        for entry in &self.monos {
            let approx_potential_savings_percent =
                (f64::from(entry.approx_potential_savings)) / (f64::from(items.size())) * 100.0;

            let total_percent = (f64::from(entry.total)) / (f64::from(items.size())) * 100.0;
            let mut rc = Record {
                generic: Some(entry.generic[..].to_string()),
                approximate_monomorphization_bloat_bytes: Some(entry.approx_potential_savings),
                approximate_monomorphization_bloat_percent: Some(approx_potential_savings_percent),
                total_size: Some(entry.total),
                total_size_percent: Some(total_percent),
                ..Default::default()
            };

            for &id in &entry.insts {
                let item = &items[id];
                let size = item.size();
                let size_percent = (f64::from(size)) / (f64::from(items.size())) * 100.0;
                rc.monomorphizations = csv::CsvRecord {
                    name: item.name().to_string(),
                    shallow_size: size,
                    shallow_size_percent: size_percent,
                    ..Default::default()
                }
            }

            wtr.serialize(rc)?;
            wtr.flush()?;
        }

        Ok(())
    }
}

/// Find bloaty monomorphizations of generic functions.
pub fn monos(items: &mut ir::Items, opts: &opt::Monos) -> Result<Box<traits::Emit>, traits::Error> {
    let mut monos = BTreeMap::new();
    for item in items.iter() {
        if let Some(generic) = item.monomorphization_of() {
            monos
                .entry(generic)
                .or_insert(BTreeSet::new())
                .insert(item.id());
        }
    }

    let mut monos: Vec<_> = monos
        .into_iter()
        .filter_map(|(generic, insts)| {
            if insts.len() <= 1 {
                return None;
            }

            let max = insts.iter().map(|id| items[*id].size()).max().unwrap();
            let total = insts.iter().map(|id| items[*id].size()).sum();
            let size_per_inst = total / (insts.len() as u32);
            let approx_potential_savings =
                cmp::min(size_per_inst * (insts.len() as u32 - 1), total - max);

            let generic = generic.to_string();

            let mut insts: Vec<_> = insts.into_iter().collect();
            insts.sort_by(|a, b| {
                let a = &items[*a];
                let b = &items[*b];
                b.size().cmp(&a.size())
            });
            insts.truncate(if opts.only_generics() {
                0
            } else {
                opts.max_monos() as usize
            });

            Some(MonosEntry {
                generic,
                insts,
                total,
                approx_potential_savings,
            })
        })
        .collect();

    monos.sort();
    monos.truncate(opts.max_generics() as usize);

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

impl traits::Emit for Diff {
    fn emit_text(
        &self,
        _items: &ir::Items,
        dest: &mut std::io::Write,
    ) -> Result<(), traits::Error> {
        let mut table = Table::with_header(vec![
            (Align::Right, "Delta Bytes".into()),
            (Align::Left, "Item".to_string()),
        ]);

        for entry in &self.deltas {
            table.add_row(vec![format!("{:+}", entry.delta), entry.name.clone()]);
        }

        write!(dest, "{}", &table)?;
        Ok(())
    }

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
}

/// Compute the diff between two sets of items.
pub fn diff(
    old_items: &mut ir::Items,
    new_items: &mut ir::Items,
    opts: &opt::Diff,
) -> Result<Box<traits::Emit>, traits::Error> {
    let old_items_by_name: BTreeMap<&str, &ir::Item> =
        old_items.iter().map(|item| (item.name(), item)).collect();
    let new_items_by_name: BTreeMap<&str, &ir::Item> =
        new_items.iter().map(|item| (item.name(), item)).collect();

    let mut deltas = vec![];

    for (name, old_item) in &old_items_by_name {
        match new_items_by_name.get(name) {
            None => deltas.push(DiffEntry {
                name: name.to_string(),
                delta: -(old_item.size() as i64),
            }),
            Some(new_item) => {
                let delta = new_item.size() as i64 - old_item.size() as i64;
                if delta != 0 {
                    deltas.push(DiffEntry {
                        name: name.to_string(),
                        delta,
                    });
                }
            }
        }
    }

    for (name, new_item) in &new_items_by_name {
        if !old_items_by_name.contains_key(name) {
            deltas.push(DiffEntry {
                name: name.to_string(),
                delta: new_item.size() as i64,
            });
        }
    }

    deltas.push(DiffEntry {
        name: "<total>".to_string(),
        delta: new_items.size() as i64 - old_items.size() as i64,
    });

    deltas.sort();
    deltas.truncate(opts.max_items() as usize);

    let diff = Diff { deltas };
    Ok(Box::new(diff) as Box<traits::Emit>)
}

#[derive(Debug)]
struct Garbage {
    items: Vec<ir::Id>,
}

impl traits::Emit for Garbage {
    fn emit_text(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut table = Table::with_header(vec![
            (Align::Right, "Bytes".to_string()),
            (Align::Right, "Size %".to_string()),
            (Align::Left, "Garbage Item".to_string()),
        ]);

        for &id in &self.items {
            let item = &items[id];
            let size = item.size();
            let size_percent = (f64::from(size)) / (f64::from(items.size())) * 100.0;
            table.add_row(vec![
                size.to_string(),
                format!("{:.2}%", size_percent),
                item.name().to_string(),
            ]);
        }

        write!(dest, "{}", &table)?;
        Ok(())
    }

    fn emit_json(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut arr = json::array(dest)?;

        for &id in &self.items {
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
    unreachable_items.truncate(opts.max_items() as usize);

    let unreachable_items: Vec<_> = unreachable_items.iter().map(|item| item.id()).collect();

    let garbage_items = Garbage {
        items: unreachable_items,
    };

    Ok(Box::new(garbage_items) as Box<traits::Emit>)
}
