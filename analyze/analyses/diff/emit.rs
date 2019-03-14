use std::io;

use csv;

use crate::formats::json;
use crate::formats::table::{Align, Table};
use twiggy_ir as ir;
use twiggy_traits as traits;

use super::Diff;

impl traits::Emit for Diff {
    #[cfg(feature = "emit_text")]
    fn emit_text(&self, _items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut table = Table::with_header(vec![
            (Align::Right, "Delta Bytes".into()),
            (Align::Left, "Item".to_string()),
        ]);

        self.deltas
            .iter()
            .map(|entry| vec![format!("{:+}", entry.delta), entry.name.clone()])
            .for_each(|row| table.add_row(row));

        write!(dest, "{}", &table)?;
        Ok(())
    }

    #[cfg(feature = "emit_json")]
    fn emit_json(&self, _items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut arr = json::array(dest)?;

        for entry in &self.deltas {
            let mut obj = arr.object()?;
            obj.field("delta_bytes", entry.delta as f64)?;
            obj.field("name", entry.name.as_str())?;
        }

        Ok(())
    }

    #[cfg(feature = "emit_csv")]
    fn emit_csv(&self, _items: &ir::Items, dest: &mut io::Write) -> Result<(), traits::Error> {
        let mut wtr = csv::Writer::from_writer(dest);

        for entry in &self.deltas {
            wtr.serialize(entry)?;
            wtr.flush()?;
        }

        Ok(())
    }
}
