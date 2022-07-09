use gimli;
use twiggy_ir as ir;
mod item_name;
mod location_attrs;

use self::item_name::item_name;
use self::location_attrs::DieLocationAttributes;

/// This type alias is used to represent an option return value for
/// a procedure that could return an Error.
type FallilbleOption<T> = anyhow::Result<Option<T>>;

pub(super) fn parse_items<R: gimli::Reader>(
    items: &mut ir::ItemsBuilder,
    dwarf: &gimli::Dwarf<R>,
    unit: &gimli::Unit<R>,
    unit_id: usize,
    entry: &gimli::DebuggingInformationEntry<R>,
    entry_id: usize,
) -> anyhow::Result<()> {
    let item: ir::Item = match entry.tag() {
        gimli::DW_TAG_subprogram => {
            if let Some(size) = DieLocationAttributes::try_from(entry)?.entity_size(dwarf, unit)? {
                let id = ir::Id::entry(unit_id, entry_id);
                let name = item_name(entry, dwarf, unit)?
                    .unwrap_or_else(|| format!("Subroutine[{}][{}]", unit_id, entry_id));
                let kind: ir::ItemKind = ir::Code::new(&name).into();
                ir::Item::new(id, &name, size as u32, kind)
            } else {
                return Ok(());
            }
        }
        _ => return Ok(()),
    };

    items.add_item(item);
    Ok(())
}

pub(super) fn parse_edges<R: gimli::Reader>(
    _items: &mut ir::ItemsBuilder,
    _entry: &gimli::DebuggingInformationEntry<R>,
) -> anyhow::Result<()> {
    // TODO: Add edges representing the call graph.
    Ok(())
}
