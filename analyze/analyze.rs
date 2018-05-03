//! Implementations of the analyses that `twiggy` runs on its IR.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate twiggy_ir as ir;
extern crate twiggy_opt as opt;
extern crate twiggy_traits as traits;

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
            0,
            &mut row,
            &opts,
            items.meta_root(),
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

            // TODO FITZGEN: this needs to do the filtering like how text
            // formatting does, but it would be ncie to push that earlier, like
            // `top` does.

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
        recursive_add_children(items, &self.opts, &self.tree, items.meta_root(), &mut obj)
    }
}

/// Compute the dominator tree for the given IR graph.
pub fn dominators(
    items: &mut ir::Items,
    opts: &opt::Dominators,
) -> Result<Box<traits::Emit>, traits::Error> {
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
                    let size_percent = (f64::from(item.size())) / (f64::from(items.size())) * 100.0;
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
}

/// Find all retaining paths for the given items.
pub fn paths(items: &mut ir::Items, opts: &opt::Paths) -> Result<Box<traits::Emit>, traits::Error> {
    items.compute_predecessors();

    let mut paths = Paths {
        items: Vec::with_capacity(opts.functions().len()),
        opts: opts.clone(),
    };

    let functions: BTreeSet<_> = opts.functions().iter().map(|s| s.as_str()).collect();

    for item in items.iter() {
        if functions.contains(item.name()) {
            paths.items.push(item.id());
        }
    }

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
}

/// Find all retaining paths for the given items.
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
