use std::collections::BTreeSet;
use std::io;

use petgraph;

use formats::json;
use formats::table::{Align, Table};
use twiggy_ir as ir;
use twiggy_opt as opt;
use twiggy_traits as traits;

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
