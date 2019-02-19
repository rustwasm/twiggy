use std::collections::BTreeMap;
use std::io;

use csv;

use formats::json;
use formats::table::{Align, Table};
use twiggy_ir as ir;
use twiggy_opt as opt;
use twiggy_traits as traits;

use crate::analyses::{dominators::DominatorTree, utils};

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
                add_text_item(items, depth, id, table);
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

        if !self.unreachable_items.is_empty() {
            let unreachable_items_cnt = self.unreachable_items.len();
            let unreachable_items_size = self
                .unreachable_items
                .iter()
                .map(|id| &items[*id])
                .map(|item| item.size())
                .sum::<u32>();
            let unreachable_items_size_percent =
                (f64::from(unreachable_items_size)) / (f64::from(items.size())) * 100.0;
            table.add_row(vec![
                unreachable_items_size.to_string(),
                format!("{:.2}%", unreachable_items_size_percent),
                format!("[{} Unreachable Items]", unreachable_items_cnt),
            ]);
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
            add_json_item(items, id, obj)?;

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

        // Scoping the borrow of `arr` so we can get another object in the next block
        let mut obj = json::object(dest)?;
        {
            let mut arr = obj.array("items")?;
            for curr_id in &self.items {
                let mut item = arr.object()?;
                recursive_add_children(items, &self.opts, &self.tree, *curr_id, &mut item)?;
            }
        }

        if !self.unreachable_items.is_empty() {
            let mut summary_obj = obj.array("summary")?;
            let mut unreachable_items_obj = summary_obj.object()?;
            let unreachable_items_cnt = self.unreachable_items.len();
            let unreachable_items_size = self
                .unreachable_items
                .iter()
                .map(|id| &items[*id])
                .map(|item| item.size())
                .sum::<u32>();
            let unreachable_items_size_percent =
                (f64::from(unreachable_items_size)) / (f64::from(items.size())) * 100.0;
            unreachable_items_obj.field(
                "name",
                format!("[{} Unreachable Items]", unreachable_items_cnt).as_ref(),
            )?;
            unreachable_items_obj.field("retained_size", unreachable_items_size)?;
            unreachable_items_obj.field("retained_size_percent", unreachable_items_size_percent)?;
        }

        Ok(())
    }

    #[cfg(feature = "emit_csv")]
    fn emit_csv(&self, items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        fn recursive_add_children(
            items: &ir::Items,
            opts: &opt::Dominators,
            dominator_tree: &BTreeMap<ir::Id, Vec<ir::Id>>,
            id: ir::Id,
            wtr: &mut csv::Writer<&mut io::Write>,
        ) -> Result<(), traits::Error> {
            add_csv_item(items, id, wtr)?;
            if let Some(children) = dominator_tree.get(&id) {
                let mut children = children.to_vec();
                children.sort_by(|a, b| items.retained_size(*b).cmp(&items.retained_size(*a)));
                for child in children {
                    recursive_add_children(items, opts, dominator_tree, child, wtr)?;
                }
            }
            Ok(())
        }

        let mut wtr = csv::Writer::from_writer(dest);
        recursive_add_children(items, &self.opts, &self.tree, items.meta_root(), &mut wtr)?;

        if !self.unreachable_items.is_empty() {
            let cnt = self.unreachable_items.len();
            let size = utils::get_unreachable_items(&items)
                .map(|item| item.size())
                .sum::<u32>();
            let size_percent = f64::from(size) / f64::from(items.size()) * 100.0;
            let rc = CsvRecord {
                id: None,
                name: format!("[{} Unreachable Items]", cnt),
                shallow_size: size,
                shallow_size_percent: size_percent,
                retained_size: size,
                retained_size_percent: size_percent,
                immediate_dominator: None,
            };
            wtr.serialize(rc)?;
            wtr.flush()?;
        }

        Ok(())
    }
}

#[cfg(feature = "emit_text")]
fn add_text_item(items: &ir::Items, depth: u32, id: ir::Id, table: &mut Table) {
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

#[cfg(feature = "emit_json")]
fn add_json_item(
    items: &ir::Items,
    id: ir::Id,
    obj: &mut json::Object,
) -> Result<(), traits::Error> {
    let item = &items[id];

    obj.field("name", item.name())?;

    let shallow_size = item.size();
    let shallow_size_percent = f64::from(shallow_size) / f64::from(items.size()) * 100.0;
    obj.field("shallow_size", shallow_size)?;
    obj.field("shallow_size_percent", shallow_size_percent)?;

    let retained_size = items.retained_size(id);
    let retained_size_percent = f64::from(retained_size) / f64::from(items.size()) * 100.0;
    obj.field("retained_size", retained_size)?;
    obj.field("retained_size_percent", retained_size_percent)?;
    Ok(())
}

#[cfg(feature = "emit_csv")]
#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct CsvRecord {
    pub id: Option<u64>,
    pub name: String,
    pub shallow_size: u32,
    pub shallow_size_percent: f64,
    pub retained_size: u32,
    pub retained_size_percent: f64,
    pub immediate_dominator: Option<u64>,
}

#[cfg(feature = "emit_csv")]
fn add_csv_item(
    items: &ir::Items,
    id: ir::Id,
    wtr: &mut csv::Writer<&mut io::Write>,
) -> Result<(), traits::Error> {
    let item = &items[id];
    let (shallow_size, shallow_size_percent) = (
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
        id: Some(item.id().serializable()),
        name: item.name().to_string(),
        shallow_size,
        shallow_size_percent,
        retained_size,
        retained_size_percent,
        immediate_dominator: Some(idom),
    };

    wtr.serialize(rc)?;
    wtr.flush()?;
    Ok(())
}
