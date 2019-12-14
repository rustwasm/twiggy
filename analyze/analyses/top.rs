use std::io;

use csv;
use serde_derive::Serialize;

use crate::formats::json;
use crate::formats::table::{Align, Table};
use twiggy_ir as ir;
use twiggy_opt as opt;
use twiggy_traits as traits;

/// The largest items found in a binary.
#[derive(Debug)]
pub struct Top {
    items: Vec<TopEntry>,
    opts: opt::Top,
}

#[derive(Debug)]
pub struct TopEntry {
    name: String,
    shallow_size: u32,
    shallow_size_percent: f64,
    retained_size: Option<u32>,
    retained_size_percent: Option<f64>,
}

impl Top {
    /// Get a list of the largest items.
    pub fn items(&self) -> &[TopEntry] {
        self.items.as_ref()
    }
}

impl traits::Emit for Top {
    #[cfg(feature = "emit_text")]
    fn emit_text(&self, _items: &ir::Items, dest: &mut dyn io::Write) -> Result<(), traits::Error> {
        // A struct used to represent a row in the table that will be emitted.
        struct TableRow {
            size: u32,
            size_percent: f64,
            name: String,
        };

        fn create_row(entry: &TopEntry) -> TableRow {
            let (size, size_percent) = if let (Some(retained_size), Some(retained_size_percent)) =
                (entry.retained_size, entry.retained_size_percent)
            {
                (retained_size, retained_size_percent)
            } else {
                (entry.shallow_size, entry.shallow_size_percent)
            };
            TableRow {
                name: entry.name.clone(),
                size,
                size_percent,
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
        self.items.iter().take(max_items).map(create_row).for_each(
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
        let remaining_rows = self.items.iter().skip(max_items).map(create_row);
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
        let all_rows = self.items.iter().map(create_row);
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

        for item in items_iter.take(max_items) {
            let mut obj = arr.object()?;
            obj.field("name", item.name.as_ref())?;

            let shallow_size = item.shallow_size;
            let size_percent = f64::from(shallow_size) / f64::from(items.size()) * 100.0;
            obj.field("shallow_size", shallow_size)?;
            obj.field("shallow_size_percent", size_percent)?;

            if let (Some(retained_size), Some(retained_size_percent)) =
                (item.retained_size, item.retained_size_percent)
            {
                obj.field("retained_size", retained_size)?;
                obj.field("retained_size_percent", retained_size_percent)?;
            }
        }

        Ok(())
    }

    #[cfg(feature = "emit_csv")]
    fn emit_csv(&self, _items: &ir::Items, dest: &mut dyn io::Write) -> Result<(), traits::Error> {
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

        for TopEntry {
            shallow_size,
            shallow_size_percent,
            retained_size,
            retained_size_percent,
            name,
        } in items_iter.take(max_items)
        {
            wtr.serialize(CsvRecord {
                name: name.clone(),
                shallow_size: *shallow_size,
                shallow_size_percent: *shallow_size_percent,
                retained_size: *retained_size,
                retained_size_percent: *retained_size_percent,
            })?;
            wtr.flush()?;
        }
        Ok(())
    }
}

// Helper function used to process an item, and return a struct
// representing a row containing its size and name.
fn create_top_entry(item: &ir::Item, items: &ir::Items, retained: bool) -> TopEntry {
    let shallow_size = item.size();
    let shallow_size_percent = (f64::from(shallow_size)) / (f64::from(items.size())) * 100.0;
    let (retained_size, retained_size_percent) = if retained {
        let retained_size = items.retained_size(item.id());
        (
            Some(retained_size),
            Some((f64::from(retained_size)) / (f64::from(items.size())) * 100.0),
        )
    } else {
        (None, None)
    };
    let name = item.name().to_string();
    TopEntry {
        shallow_size,
        shallow_size_percent,
        retained_size,
        retained_size_percent,
        name,
    }
}

/// Run the `top` analysis on the given IR items.
pub fn top(items: &mut ir::Items, opts: &opt::Top) -> Result<Top, traits::Error> {
    if opts.retaining_paths() {
        return Err(traits::Error::with_msg(
            "retaining paths are not yet implemented",
        ));
    }

    if opts.retained() {
        items.compute_retained_sizes();
    }

    let mut top_items: Vec<TopEntry> = items
        .iter()
        .filter(|item| item.id() != items.meta_root())
        .map(|item: &ir::Item| create_top_entry(item, &items, opts.retained()))
        .collect();

    top_items.sort_by(|a, b| {
        if opts.retained() {
            b.retained_size.unwrap().cmp(&a.retained_size.unwrap())
        } else {
            b.shallow_size.cmp(&a.shallow_size)
        }
    });

    Ok(Top {
        items: top_items,
        opts: opts.clone(),
    })
}
