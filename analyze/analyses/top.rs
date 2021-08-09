use std::io;

use csv;
use serde_derive::Serialize;

use crate::formats::json;
use crate::formats::table::{Align, Table};
use twiggy_ir as ir;
use twiggy_opt as opt;
use twiggy_traits as traits;

struct Top {
    items: Vec<ir::Id>,
    opts: opt::Top,
}

impl traits::Emit for Top {
    #[cfg(feature = "emit_text")]
    fn emit_text(&self, items: &ir::Items, dest: &mut dyn io::Write) -> Result<(), traits::Error> {
        // A struct used to represent a row in the table that will be emitted.
        struct TableRow {
            size: u32,
            size_percent: f64,
            name: String,
        }

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
        }

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
    fn emit_json(&self, items: &ir::Items, dest: &mut dyn io::Write) -> Result<(), traits::Error> {
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
    fn emit_csv(&self, items: &ir::Items, dest: &mut dyn io::Write) -> Result<(), traits::Error> {
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
pub fn top(items: &mut ir::Items, opts: &opt::Top) -> Result<Box<dyn traits::Emit>, traits::Error> {
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

    Ok(Box::new(top) as Box<_>)
}
