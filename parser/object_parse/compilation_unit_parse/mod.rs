use gimli;
use twiggy_ir as ir;
use twiggy_traits as traits;

use super::die_parse;

pub(super) fn parse_items<R: gimli::Reader>(
    items: &mut ir::ItemsBuilder,
    dwarf: &gimli::Dwarf<R>,
    unit: &gimli::Unit<R>,
    unit_id: usize,
) -> Result<(), traits::Error> {
    // Initialize an entry ID counter.
    let mut entry_id = 0;

    // Create an entries cursor, and move it to the root.
    let mut die_cursor = unit.entries();

    if die_cursor.next_dfs()?.is_none() {
        let e = traits::Error::with_msg(
            "Unexpected error while traversing debugging information entries.",
        );
        return Err(e);
    }

    // Parse the contained debugging information entries in depth-first order.
    let mut depth = 0;
    while let Some((delta, entry)) = die_cursor.next_dfs()? {
        // Update depth value, and break out of the loop when we
        // return to the original starting position.
        depth += delta;
        if depth <= 0 {
            break;
        }

        die_parse::parse_items(items, dwarf, unit, unit_id, entry, entry_id)?;
        entry_id += 1;
    }

    Ok(())
}

pub(super) fn parse_edges<R: gimli::Reader>(
    items: &mut ir::ItemsBuilder,
    unit: &gimli::Unit<R>,
    unit_id: usize,
) -> Result<(), traits::Error> {
    // Initialize an entry ID counter.
    let mut entry_id = 0;

    // Create an entries cursor, and move it to the root.
    let mut die_cursor = unit.entries();

    if die_cursor.next_dfs()?.is_none() {
        let e = traits::Error::with_msg(
            "Unexpected error while traversing debugging information entries.",
        );
        return Err(e);
    }

    // Parse the contained debugging information entries in depth-first order.
    let mut depth = 0;
    while let Some((delta, entry)) = die_cursor.next_dfs()? {
        // Update depth value, and break out of the loop when we
        // return to the original starting position.
        depth += delta;
        if depth <= 0 {
            break;
        }

        let _ir_id = ir::Id::entry(unit_id, entry_id);
        die_parse::parse_edges(items, entry)?;
        entry_id += 1;
    }

    Ok(())
}
