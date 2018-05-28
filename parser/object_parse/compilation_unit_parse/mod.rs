use gimli;
use ir;
use traits;

use super::die_parse::DIEItemsExtra;
use super::Parse;

pub struct CompUnitItemsExtra<'input, R>
where
    R: 'input + gimli::Reader,
{
    pub unit_id: usize,
    pub debug_abbrev: gimli::DebugAbbrev<R>,
    pub debug_str: gimli::DebugStr<R>,
    pub debug_types: gimli::DebugTypes<R>,
    pub rnglists: &'input gimli::RangeLists<R>,
}

pub struct CompUnitEdgesExtra<R>
where
    R: gimli::Reader,
{
    pub unit_id: usize,
    pub debug_abbrev: gimli::DebugAbbrev<R>,
}

impl<'input, R> Parse<'input> for gimli::CompilationUnitHeader<R, R::Offset>
where
    R: 'input + gimli::Reader,
{
    type ItemsExtra = CompUnitItemsExtra<'input, R>;

    fn parse_items(
        &self,
        items: &mut ir::ItemsBuilder,
        extra: Self::ItemsExtra,
    ) -> Result<(), traits::Error> {
        // Destructure the extra information needed to parse items in the unit.
        let Self::ItemsExtra {
            unit_id,
            debug_abbrev,
            debug_str,
            debug_types,
            rnglists,
        } = extra;

        // Get the size of addresses in this type-unit, initialize an entry ID counter.
        let addr_size: u8 = self.address_size();
        let dwarf_version: u16 = self.version();
        let mut entry_id = 0;

        // Find the abbreviations associated with this compilation unit.
        // Use the abbreviations to create an entries cursor, and move it to the root.
        let abbrevs = self
            .abbreviations(&debug_abbrev)
            .expect("Could not find abbreviations");
        let mut die_cursor = self.entries(&abbrevs);
        assert!(die_cursor.next_dfs().unwrap().is_some());

        // Parse the contained debugging information entries in depth-first order.
        let mut depth = 0;
        while let Some((delta, entry)) = die_cursor.next_dfs()? {
            // Update depth value, and break out of the loop when we
            // return to the original starting position.
            depth += delta;
            assert!(depth >= 0);
            if depth <= 0 {
                break;
            }

            let die_extra = DIEItemsExtra {
                entry_id,
                unit_id,
                addr_size,
                dwarf_version,
                debug_str: &debug_str,
                debug_types: &debug_types,
                rnglists,
                comp_unit: self,
            };
            entry.parse_items(items, die_extra)?;
            entry_id += 1;
        }

        Ok(())
    }

    type EdgesExtra = CompUnitEdgesExtra<R>;

    fn parse_edges(
        &self,
        items: &mut ir::ItemsBuilder,
        extra: Self::EdgesExtra,
    ) -> Result<(), traits::Error> {
        let Self::EdgesExtra {
            unit_id,
            debug_abbrev,
        } = extra;

        // Initialize an entry ID counter.
        let mut entry_id = 0;

        // Find the abbreviations associated with this compilation unit.
        // Use the abbreviations to create an entries cursor, and move it to the root.
        let abbrevs = self
            .abbreviations(&debug_abbrev)
            .expect("Could not find abbreviations");
        let mut die_cursor = self.entries(&abbrevs);
        assert!(die_cursor.next_dfs().unwrap().is_some());

        // Parse the contained debugging information entries in depth-first order.
        let mut depth = 0;
        while let Some((delta, entry)) = die_cursor.next_dfs()? {
            // Update depth value, and break out of the loop when we
            // return to the original starting position.
            depth += delta;
            assert!(depth >= 0);
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
