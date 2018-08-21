use std::io;
use std::iter;

use csv;

use formats::json;
use formats::table::{Align, Table};
use twiggy_ir as ir;
use twiggy_opt as opt;
use twiggy_traits as traits;

use super::paths_entry::PathsEntry;
use super::Paths;

impl traits::Emit for Paths {
    #[cfg(feature = "emit_text")]
    fn emit_text(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        /// This structure represents a row in the emitted table. Size, and size
        /// percentage are only shown for the top-most rows.
        struct TableRow {
            size: Option<u32>,
            size_percent: Option<f64>,
            name: String,
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
                })).chain(iter::once(name))
                .fold(
                    String::with_capacity(depth as usize * 4 + name.len()),
                    |mut res, s| {
                        res.push_str(s);
                        res
                    },
                )
        }

        /// Process a given path entry, and return an iterator of table rows,
        /// representing its related call paths, according to the given options.
        fn process_entry<'a>(
            entry: &'a PathsEntry,
            depth: u32,
            paths: usize,
            items: &'a ir::Items,
            opts: &'a opt::Paths,
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
                let children_iter =
                    entry
                        .children
                        .iter()
                        .take(paths)
                        .flat_map(move |child_entry| {
                            process_entry(child_entry, depth + 1, paths, &items, &opts)
                        });
                Box::new(row_iter.chain(children_iter))
            } else if depth == opts.max_depth() {
                Box::new(row_iter)
            // FIXUP: Create a summary row, and chain it to the row iterator.
            // let child_count = entry.children.iter().map(|c| c.count()).sum::<u32>();
            // if child_count > 0 {
            //     let summary_row = iter::once(TableRow {
            //         name: get_indented_name(
            //             &format!("... and {} more.", child_count),
            //             depth + 1,
            //             opts.descending(),
            //         ),
            //         size: None,
            //         size_percent: None,
            //     });
            //     Box::new(row_iter.chain(summary_row))
            // } else {
            //     Box::new(row_iter)
            // }
            } else {
                // If we are beyond the maximum depth, return an empty iterator.
                Box::new(iter::empty())
            }
        }

        // Process each entry and its children, map these items into table rows,
        // and fold each of the resulting rows into a table.
        let table = self
            .entries
            .iter()
            .flat_map(|entry| {
                process_entry(entry, 0, self.opts.max_paths() as usize, &items, &self.opts)
            }).map(
                |TableRow {
                     size,
                     size_percent,
                     name,
                 }| {
                    vec![
                        size.map(|size| size.to_string()).unwrap_or("".to_string()),
                        size_percent
                            .map(|size_percent| format!("{:.2}%", size_percent))
                            .unwrap_or("".to_string()),
                        name,
                    ]
                },
            ).fold(
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
        // Process a paths entry, by adding its name and size to the given JSON object.
        fn process_entry(
            entry: &PathsEntry,
            obj: &mut json::Object,
            depth: u32,
            paths: usize,
            items: &ir::Items,
            opts: &opt::Paths,
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
            } else if depth == opts.max_depth() && !entry.children.is_empty() {
                // FIXUP: This will create a summary element, when ready.
                // let mut obj = callers.object()?;
                // let rem_cnt = entry.children.iter().map(|c| c.count()).sum::<u32>();
                // let rem_name = format!("... and {} more.", rem_cnt);
                // obj.field("name", rem_name.as_str())?;
            }

            Ok(())
        }

        // Add each path entry to a JSON array.
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
        // This structure represents a row in the CSV output.
        #[derive(Serialize, Debug)]
        #[serde(rename_all = "PascalCase")]
        struct CsvRecord {
            name: String,
            shallow_size: u32,
            shallow_size_percent: f64,
            path: Option<String>,
        }

        // Given an entry, return the value for the record's `path` field.
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

        // Process a given entry and its children, returning an iterator of CSV records.
        fn process_entry<'a>(
            entry: &'a PathsEntry,
            depth: u32,
            paths: usize,
            items: &'a ir::Items,
            opts: &'a opt::Paths,
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
                let children_iter =
                    entry
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

        // First, initialize a CSV writer. Then, process each entry.
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
