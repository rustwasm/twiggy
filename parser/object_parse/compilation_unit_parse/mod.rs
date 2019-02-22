use gimli;
use ir;
use traits;

use super::die_parse::DieItemsExtra;
use super::Parse;

pub struct CompUnitItemsExtra<'input, R>
where
    R: 'input + gimli::Reader,
{
    pub unit_id: usize,
    pub dwarf: &'input gimli::Dwarf<R>,
}

pub struct CompUnitEdgesExtra {
    pub unit_id: usize,
}

impl<'input, R> Parse<'input> for gimli::Unit<R>
where
    R: 'input + gimli::Reader,
{
    type ItemsExtra = CompUnitItemsExtra<'input, R>;

    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        extra: Self::ItemsExtra,
    ) -> Result<(), traits::Error> {
        // Destructure the extra information needed to parse items in the unit.
        let Self::ItemsExtra {
            unit_id,
            dwarf,
        } = extra;

        // Initialize an entry ID counter.
        let mut entry_id = 0;

        // Create an entries cursor, and move it to the root.
        let mut die_cursor = self.entries();

        if die_cursor.next_dfs()?.is_none() {
            let e = traits::Error::with_msg(
                "Unexpected error while traversing debugging information entries.",
            );
            return Err(e);
        }

        // Parse the contained debugging information entries in depth-first order.
        let mut depth = 0;
        while let Some((delta, mut entry)) = die_cursor.next_dfs()? {
            // Update depth value, and break out of the loop when we
            // return to the original starting position.
            depth += delta;
            if depth <= 0 {
                break;
            }

            let die_extra = DieItemsExtra {
                entry_id,
                unit_id,
                dwarf,
                unit: self,
            };
            entry.parse_items(items, die_extra)?;
            entry_id += 1;
        }

        Ok(())
    }

    type EdgesExtra = CompUnitEdgesExtra;

    fn parse_edges(
        &mut self,
        items: &mut ir::ItemsBuilder,
        extra: Self::EdgesExtra,
    ) -> Result<(), traits::Error> {
        let Self::EdgesExtra {
            unit_id,
        } = extra;

        // Initialize an entry ID counter.
        let mut entry_id = 0;

        // Create an entries cursor, and move it to the root.
        let mut die_cursor = self.entries();

        if die_cursor.next_dfs()?.is_none() {
            let e = traits::Error::with_msg(
                "Unexpected error while traversing debugging information entries.",
            );
            return Err(e);
        }

        // Parse the contained debugging information entries in depth-first order.
        let mut depth = 0;
        while let Some((delta, mut entry)) = die_cursor.next_dfs()? {
            // Update depth value, and break out of the loop when we
            // return to the original starting position.
            depth += delta;
            if depth <= 0 {
                break;
            }

            let _ir_id = ir::Id::entry(unit_id, entry_id);
            entry.parse_edges(items, ())?;
            entry_id += 1;
        }

        Ok(())
    }
}
