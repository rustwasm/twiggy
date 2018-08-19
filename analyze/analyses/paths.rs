use std::collections::BTreeSet;
use std::io;

use csv;
use regex;

use formats::json;
use formats::table::{Align, Table};
use twiggy_ir as ir;
use twiggy_opt as opt;
use twiggy_traits as traits;

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

    // Initialize the collection of Id values whose retaining paths we will emit.
    let opts = opts.clone();
    let items = get_items(&items, &opts)?;
    let paths = Paths { items, opts };

    Ok(Box::new(paths) as Box<traits::Emit>)
}

/// This helper function is used to collect ir::Id values for the `items` member
/// of the `Paths` object, based on the given options.
pub fn get_items(items: &ir::Items, opts: &opt::Paths) -> Result<Vec<ir::Id>, traits::Error> {
    // Collect Id's if no arguments are given and we are ascending the retaining paths.
    let get_functions_default = || -> Vec<ir::Id> {
        let mut sorted_items = items
            .iter()
            .filter(|item| item.id() != items.meta_root())
            .collect::<Vec<_>>();
        sorted_items.sort_by(|a, b| b.size().cmp(&a.size()));
        sorted_items.iter().map(|item| item.id()).collect()
    };

    // Collect Id's if no arguments are given and we are descending the retaining paths.
    let get_functions_default_desc = || -> Vec<ir::Id> {
        let mut roots = items
            .neighbors(items.meta_root())
            .map(|id| &items[id])
            .collect::<Vec<_>>();
        roots.sort_by(|a, b| b.size().cmp(&a.size()));
        roots.into_iter().map(|item| item.id()).collect()
    };

    // Collect Id's if arguments were given that should be used as regular expressions.
    let get_regexp_matches = || -> Result<Vec<ir::Id>, traits::Error> {
        let regexps = regex::RegexSet::new(opts.functions())?;
        let matches = items
            .iter()
            .filter(|item| regexps.is_match(&item.name()))
            .map(|item| item.id())
            .collect();
        Ok(matches)
    };

    // Collect Id's if arguments were given that should be used as exact names.
    let get_exact_matches = || -> Vec<ir::Id> {
        opts.functions()
            .iter()
            .filter_map(|s| items.get_item_by_name(s))
            .map(|item| item.id())
            .collect()
    };

    // Collect the starting positions based on the relevant options given.
    // If arguments were given, search for matches depending on whether or
    // not these should be treated as regular expressions. Otherwise, collect
    // the starting positions based on the direction we will be traversing.
    let args_given = !opts.functions().is_empty();
    let using_regexps = opts.using_regexps();
    let descending = opts.descending();
    let res = match (args_given, using_regexps, descending) {
        (true, true, _) => get_regexp_matches()?,
        (true, false, _) => get_exact_matches(),
        (false, _, true) => get_functions_default_desc(),
        (false, _, false) => get_functions_default(),
    };

    Ok(res)
}
