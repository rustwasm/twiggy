use std::io;

use csv;

use crate::analyses::paths::Paths;
use crate::formats::json;
use crate::formats::table::{Align, Table};
use twiggy_ir as ir;
use twiggy_traits as traits;

impl traits::Emit for Paths {
    #[cfg(feature = "emit_text")]
    fn emit_text(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        use self::emit_text_helpers::{process_entry, TableRow};

        // Flat map each entry and its children into a sequence of table rows.
        // Convert these `TableRow` objects into vectors of strings, and add
        // each of these to the table before writing the table to `dest`.
        let table = self
            .entries
            .iter()
            .flat_map(|entry| {
                process_entry(entry, 0, self.opts.max_paths() as usize, &items, &self.opts)
            })
            .map(
                |TableRow {
                     size,
                     size_percent,
                     name,
                 }| {
                    vec![
                        size.map(|size| size.to_string())
                            .unwrap_or_else(String::new),
                        size_percent
                            .map(|size_percent| format!("{:.2}%", size_percent))
                            .unwrap_or_else(String::new),
                        name,
                    ]
                },
            )
            .fold(
                Table::with_header(vec![
                    (Align::Right, "Shallow Bytes".to_string()),
                    (Align::Right, "Shallow %".to_string()),
                    (Align::Left, "Retaining Paths".to_string()),
                ]),
                |mut table, row| {
                    table.add_row(row);
                    table
                },
            );

        write!(dest, "{}", table)?;
        Ok(())
    }

    #[cfg(feature = "emit_json")]
    fn emit_json(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        use self::emit_json_helpers::process_entry;

        // Initialize a JSON array. For each path entry, add a object to the
        // array, and add that entry's information to the new JSON object.
        let mut arr = json::array(dest)?;
        for entry in &self.entries {
            let mut obj = arr.object()?;
            process_entry(
                entry,
                &mut obj,
                0,
                self.opts.max_paths() as usize,
                items,
                &self.opts,
            )?;
        }

        Ok(())
    }

    #[cfg(feature = "emit_csv")]
    fn emit_csv(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        use self::emit_csv_helpers::process_entry;

        // First, initialize a CSV writer. Then, flat map each entry and its
        // children into a sequence of `CsvRecord` objects. Send each record
        // to the CSV writer to be serialized.
        let mut wtr = csv::Writer::from_writer(dest);
        for record in self.entries.iter().flat_map(|entry| {
            process_entry(entry, 0, self.opts.max_paths() as usize, &items, &self.opts)
        }) {
            wtr.serialize(record)?;
            wtr.flush()?;
        }

        Ok(())
    }
}

/// This module contains helper functions and structs used by the `emit_text`
/// method in Path's implementation of the `traits::Emit` trait.
mod emit_text_helpers {
    use crate::analyses::paths::paths_entry::PathsEntry;
    use std::iter;
    use twiggy_ir::Items;
    use twiggy_opt::Paths;

    /// This structure represents a row in the emitted text table. Size, and size
    /// percentage are only shown for the top-most rows.
    pub(super) struct TableRow {
        pub size: Option<u32>,
        pub size_percent: Option<f64>,
        pub name: String,
    }

    /// Process a given path entry, and return an iterator of table rows,
    /// representing its related call paths, according to the given options.
    pub(super) fn process_entry<'a>(
        entry: &'a PathsEntry,
        depth: u32,
        paths: usize,
        items: &'a Items,
        opts: &'a Paths,
    ) -> Box<dyn Iterator<Item = TableRow> + 'a> {
        // Get the row's name and size columns using the current depth.
        let name = get_indented_name(&entry.name, depth, opts.descending());
        let (size, size_percent) = if depth == 0 {
            (
                Some(entry.size),
                Some(f64::from(entry.size) / f64::from(items.size()) * 100.0),
            )
        } else {
            (None, None)
        };

        // Create an iterator containing the current entry's table row.
        let row_iter = iter::once(TableRow {
            size,
            size_percent,
            name,
        });

        if depth < opts.max_depth() {
            // Process each child entry, and chain together the resulting iterators.
            let children_iter = entry
                .children
                .iter()
                .take(paths)
                .flat_map(move |child_entry| {
                    process_entry(child_entry, depth + 1, paths, &items, &opts)
                });
            Box::new(row_iter.chain(children_iter))
        } else if depth == opts.max_depth() {
            // TODO: Create a summary row, and chain it to the row iterator.
            Box::new(row_iter)
        } else {
            // If we are beyond the maximum depth, return an empty iterator.
            Box::new(iter::empty())
        }
    }

    /// Given the name of an item, its depth, and the traversal direction,
    /// return an indented version of the name for its corresponding table row.
    fn get_indented_name(name: &str, depth: u32, descending: bool) -> String {
        (1..depth)
            .map(|_| "    ")
            .chain(iter::once(if depth > 0 && descending {
                "  ↳ "
            } else if depth > 0 {
                "  ⬑ "
            } else {
                ""
            }))
            .chain(iter::once(name))
            .fold(
                String::with_capacity(depth as usize * 4 + name.len()),
                |mut res, s| {
                    res.push_str(s);
                    res
                },
            )
    }
}

/// This module contains helper functions and structs used by the `emit_json`
/// method in Path's implementation of the `traits::Emit` trait.
mod emit_json_helpers {
    use crate::analyses::paths::paths_entry::PathsEntry;
    use crate::formats::json::Object;
    use std::io;
    use twiggy_ir::Items;
    use twiggy_opt::Paths;

    // Process a paths entry, by adding its name and size to the given JSON object.
    pub(super) fn process_entry(
        entry: &PathsEntry,
        obj: &mut Object,
        depth: u32,
        paths: usize,
        items: &Items,
        opts: &Paths,
    ) -> io::Result<()> {
        let PathsEntry {
            name,
            size,
            children,
        } = entry;
        obj.field("name", name.as_str())?;
        obj.field("shallow_size", *size)?;
        let size_percent = f64::from(*size) / f64::from(items.size()) * 100.0;
        obj.field("shallow_size_percent", size_percent)?;

        let mut callers = obj.array("callers")?;
        if depth < opts.max_depth() {
            for child in children.iter().take(paths) {
                let mut obj = callers.object()?;
                process_entry(child, &mut obj, depth + 1, paths, items, &opts)?;
            }
        }

        Ok(())
    }
}

/// This module contains helper functions and structs used by the `emit_csv`
/// method in Path's implementation of the `traits::Emit` trait.
mod emit_csv_helpers {
    use crate::analyses::paths::paths_entry::PathsEntry;
    use serde_derive::Serialize;
    use std::iter;
    use twiggy_ir::Items;
    use twiggy_opt::Paths;

    /// This structure represents a row in the CSV output.
    #[derive(Serialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    pub(super) struct CsvRecord {
        pub name: String,
        pub shallow_size: u32,
        pub shallow_size_percent: f64,
        pub path: Option<String>,
    }

    // Process a given entry and its children, returning an iterator of CSV records.
    pub(super) fn process_entry<'a>(
        entry: &'a PathsEntry,
        depth: u32,
        paths: usize,
        items: &'a Items,
        opts: &'a Paths,
    ) -> Box<dyn Iterator<Item = CsvRecord> + 'a> {
        let name = entry.name.clone();
        let shallow_size = entry.size;
        let shallow_size_percent = f64::from(entry.size) / f64::from(items.size()) * 100.0;
        let path = get_path(entry);

        // Create an iterator containing the current entry's CSV record.
        let record_iter = iter::once(CsvRecord {
            name,
            shallow_size,
            shallow_size_percent,
            path,
        });

        if depth < opts.max_depth() {
            // Process each child entry, and chain together the resulting iterators.
            let children_iter = entry
                .children
                .iter()
                .take(paths)
                .flat_map(move |child_entry| {
                    process_entry(child_entry, depth + 1, paths, &items, &opts)
                });
            Box::new(record_iter.chain(children_iter))
        } else if depth == opts.max_depth() {
            // Create a summary row, and chain it to the row iterator.
            Box::new(record_iter)
        } else {
            // If we are beyond the maximum depth, return an empty iterator.
            Box::new(iter::empty())
        }
    }

    // Given a path entry, return the value for its corresponding CsvRecord's `path` field.
    fn get_path(entry: &PathsEntry) -> Option<String> {
        if entry.children.is_empty() {
            None
        } else {
            Some(
                entry
                    .children
                    .iter()
                    .map(|child| child.name.as_str())
                    .chain(iter::once(entry.name.as_str()))
                    .collect::<Vec<_>>()
                    .join(" -> "),
            )
        }
    }
}
