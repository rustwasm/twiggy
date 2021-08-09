use std::io;
use std::iter;

use csv;
use serde_derive::Serialize;

use crate::formats::json;
use crate::formats::table::{Align, Table};
use twiggy_ir as ir;
use twiggy_traits as traits;

use super::entry::MonosEntry;
use super::Monos;

impl traits::Emit for Monos {
    #[cfg(feature = "emit_text")]
    fn emit_text(&self, items: &ir::Items, dest: &mut dyn io::Write) -> Result<(), traits::Error> {
        struct TableRow {
            bloat: Option<u32>,
            bloat_percent: Option<f64>,
            size: u32,
            size_percent: f64,
            name: String,
        }

        // Given an entry representing a generic function and its various
        // monomorphizations, return a vector of table rows.
        fn process_entry<'a>(
            entry: &'a MonosEntry,
            total_size: f64,
        ) -> impl Iterator<Item = TableRow> + 'a {
            let MonosEntry {
                name,
                insts,
                size,
                bloat,
            } = entry;

            let get_size_percent = move |x: u32| f64::from(x) / total_size * 100.0;

            iter::once(TableRow {
                bloat: Some(*bloat),
                bloat_percent: Some(get_size_percent(*bloat)),
                size: *size,
                size_percent: get_size_percent(*size),
                name: name.to_string(),
            })
            .chain(insts.iter().map(move |(name, size)| TableRow {
                bloat: None,
                bloat_percent: None,
                size: *size,
                size_percent: get_size_percent(*size),
                name: format!("    {}", name),
            }))
        }

        let mut table = Table::with_header(vec![
            (Align::Right, "Apprx. Bloat Bytes".into()),
            (Align::Right, "Apprx. Bloat %".into()),
            (Align::Right, "Bytes".into()),
            (Align::Right, "%".into()),
            (Align::Left, "Monomorphizations".to_string()),
        ]);

        for TableRow {
            bloat,
            bloat_percent,
            size,
            size_percent,
            name,
        } in self
            .monos
            .iter()
            .flat_map(|mono| process_entry(mono, f64::from(items.size())))
        {
            table.add_row(vec![
                bloat.map(|b| b.to_string()).unwrap_or_default(),
                bloat_percent
                    .map(|b| format!("{:.2}%", b))
                    .unwrap_or_default(),
                size.to_string(),
                format!("{:.2}%", size_percent),
                name.clone(),
            ]);
        }
        write!(dest, "{}", &table)?;
        Ok(())
    }

    #[cfg(feature = "emit_json")]
    fn emit_json(&self, items: &ir::Items, dest: &mut dyn io::Write) -> Result<(), traits::Error> {
        // Given an entry representing a generic function and its various
        // monomorphizations, add its information to the given JSON object.
        fn process_entry(
            entry: &MonosEntry,
            obj: &mut json::Object,
            total_size: f64,
        ) -> Result<(), traits::Error> {
            let get_size_percent = |size: u32| (f64::from(size)) / total_size * 100.0;
            let MonosEntry {
                name,
                insts,
                size,
                bloat,
            } = entry;
            obj.field("generic", name.as_str())?;
            obj.field("approximate_monomorphization_bloat_bytes", *bloat)?;
            obj.field(
                "approximate_monomorphization_bloat_percent",
                get_size_percent(*bloat),
            )?;
            obj.field("total_size", *size)?;
            obj.field("total_size_percent", get_size_percent(*size))?;
            let mut monos = obj.array("monomorphizations")?;
            for (name, size, size_percent) in insts
                .iter()
                .map(|(name, size)| (name, size, get_size_percent(*size)))
            {
                let mut obj = monos.object()?;
                obj.field("name", name.as_str())?;
                obj.field("shallow_size", *size)?;
                obj.field("shallow_size_percent", size_percent)?;
            }
            Ok(())
        }

        let items_size = f64::from(items.size());
        let mut arr = json::array(dest)?;
        for entry in &self.monos {
            let mut obj = arr.object()?;
            process_entry(entry, &mut obj, items_size)?;
        }

        Ok(())
    }

    #[cfg(feature = "emit_csv")]
    fn emit_csv(&self, items: &ir::Items, dest: &mut dyn io::Write) -> Result<(), traits::Error> {
        // Calculate the total size of the collection of items, and define a
        // helper closure to calculate a percent value for a given u32 size.
        let items_size = f64::from(items.size());
        let get_size_percent = |size: u32| (f64::from(size)) / items_size * 100.0;

        #[derive(Debug, Default, Serialize)]
        #[serde(rename_all = "PascalCase")]
        struct Record {
            generic: Option<String>,
            approximate_monomorphization_bloat_bytes: Option<u32>,
            approximate_monomorphization_bloat_percent: Option<f64>,
            total_size: Option<u32>,
            total_size_percent: Option<f64>,
            monomorphizations: Option<String>,
        }

        // Given a single `MonosEntry` object, create a record object.
        let process_entry = |entry: &MonosEntry| -> Record {
            let monos = entry
                .insts
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>();
            Record {
                generic: Some(entry.name.clone()),
                approximate_monomorphization_bloat_bytes: Some(entry.bloat),
                approximate_monomorphization_bloat_percent: Some(get_size_percent(entry.bloat)),
                total_size: Some(entry.size),
                total_size_percent: Some(get_size_percent(entry.size)),
                monomorphizations: Some(monos.join(", ")),
            }
        };

        // Create a CSV writer and iterate through the monomorphization entries.
        // Process each record and pass it to the destination to be serialized.
        let mut wtr = csv::Writer::from_writer(dest);
        for entry in &self.monos {
            let record = process_entry(entry);
            wtr.serialize(record)?;
            wtr.flush()?;
        }
        Ok(())
    }
}
