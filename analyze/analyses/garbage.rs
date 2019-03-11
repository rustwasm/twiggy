use std::collections::BTreeSet;
use std::io;

use petgraph::visit::Walker;

use crate::formats::json;
use crate::formats::table::{Align, Table};
use twiggy_ir as ir;
use twiggy_opt as opt;
use twiggy_traits as traits;

#[derive(Debug)]
struct Garbage {
    items: Vec<ir::Id>,
    data_segments: Vec<ir::Id>,
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
        let items_iter = self.items.iter().map(|id| &items[*id]);

        for item in items_iter.clone().take(self.limit) {
            let size = item.size();
            let size_percent = (f64::from(size)) / (f64::from(items.size())) * 100.0;
            table.add_row(vec![
                size.to_string(),
                format!("{:.2}%", size_percent),
                item.name().to_string(),
            ]);
        }

        match items_iter
            .clone()
            .skip(self.limit)
            .fold((0, 0), |(size, cnt), item| (size + item.size(), cnt + 1))
        {
            (size, cnt) if cnt > 0 => {
                let size_percent = f64::from(size) / f64::from(items.size()) * 100.0;
                table.add_row(vec![
                    size.to_string(),
                    format!("{:.2}%", size_percent),
                    format!("... and {} more", cnt),
                ]);
            }
            _ => {}
        }

        let total_size: u32 = items_iter.map(|item| item.size()).sum();
        let total_percent = (f64::from(total_size)) / (f64::from(items.size())) * 100.0;
        table.add_row(vec![
            total_size.to_string(),
            format!("{:.2}%", total_percent),
            format!("Σ [{} Total Rows]", self.items.len()),
        ]);

        if !self.data_segments.is_empty() {
            let total_size: u32 = self.data_segments.iter().map(|&id| items[id].size()).sum();
            let size_percent = f64::from(total_size) / f64::from(items.size()) * 100.0;
            table.add_row(vec![
                total_size.to_string(),
                format!("{:.2}%", size_percent),
                format!(
                    "{} potential false-positive data segments",
                    self.data_segments.len()
                ),
            ]);
        }

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

        let (total_size, total_cnt) = self
            .items
            .iter()
            .skip(self.limit)
            .map(|id| &items[*id])
            .fold((0, 0), |(size, cnt), item| (size + item.size(), cnt + 1));
        if total_cnt > 0 {
            let name = format!("... and {} more", total_cnt);
            let total_size_percent = (f64::from(total_size)) / (f64::from(items.size())) * 100.0;
            let mut obj = arr.object()?;
            obj.field("name", name.as_str())?;
            obj.field("bytes", total_size)?;
            obj.field("size_percent", total_size_percent)?;
        }

        // Scoping the borrow of `arr` so we can get another object in the next block
        {
            let total_name = format!("Σ [{} Total Rows]", self.items.len());
            let total_size: u32 = self.items.iter().map(|&id| items[id].size()).sum();
            let total_size_percent = (f64::from(total_size)) / (f64::from(items.size())) * 100.0;
            let mut obj = arr.object()?;
            obj.field("name", total_name.as_str())?;
            obj.field("bytes", total_size)?;
            obj.field("size_percent", total_size_percent)?;
        }

        if !self.data_segments.is_empty() {
            let name = format!(
                "{} potential false-positive data segments",
                self.data_segments.len()
            );
            let size: u32 = self.data_segments.iter().map(|&id| items[id].size()).sum();
            let size_percent = f64::from(size) / f64::from(items.size()) * 100.0;

            let mut obj = arr.object()?;
            obj.field("name", name.as_str())?;
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
    let mut unreachable_items = get_unreachable_items(&items).collect::<Vec<_>>();
    unreachable_items.sort_by(|a, b| b.size().cmp(&a.size()));

    // Split the items into two categories if necessary
    let (data_segments, items_non_data) = if opts.show_data_segments() {
        (
            vec![],
            unreachable_items.iter().map(|item| item.id()).collect(),
        )
    } else {
        (
            unreachable_items
                .iter()
                .filter(|item| item.kind().is_data())
                .map(|item| item.id())
                .collect(),
            unreachable_items
                .iter()
                .filter(|item| !item.kind().is_data())
                .map(|item| item.id())
                .collect(),
        )
    };

    let garbage_items = Garbage {
        items: items_non_data,
        data_segments,
        limit: opts.max_items() as usize,
    };

    Ok(Box::new(garbage_items) as Box<traits::Emit>)
}

pub(crate) fn get_unreachable_items(items: &ir::Items) -> impl Iterator<Item = &ir::Item> {
    let reachable_items = petgraph::visit::Dfs::new(items, items.meta_root())
        .iter(&items)
        .collect::<BTreeSet<ir::Id>>();
    items
        .iter()
        .filter(move |item| !reachable_items.contains(&item.id()))
}
