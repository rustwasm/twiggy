use gimli;
use ir;
use traits;

use super::Parse;

mod item_kind;
mod item_name;
mod location_attrs;

use self::item_kind::item_kind;
use self::item_name::item_name;
use self::location_attrs::DieLocationAttributes;

/// This type alias is used to represent an option return value for
/// a procedure that could return an Error.
type FallilbleOption<T> = Result<Option<T>, traits::Error>;

/// This struct represents the extra items required by the Parse trait's
/// `parse_items` method. This is constructed by the compilation unit's
/// own implementation of `parse_items`.
pub struct DIEItemsExtra<'unit, R>
where
    R: 'unit + gimli::Reader,
{
    pub entry_id: usize,
    pub unit_id: usize,
    pub addr_size: u8,
    pub dwarf_version: u16,
    pub debug_str: &'unit gimli::DebugStr<R>,
    pub debug_types: &'unit gimli::DebugTypes<R>,
    pub rnglists: &'unit gimli::RangeLists<R>,
    pub comp_unit: &'unit gimli::CompilationUnitHeader<R, <R as gimli::Reader>::Offset>,
}

impl<'abbrev, 'unit, R> Parse<'unit>
    for gimli::DebuggingInformationEntry<'abbrev, 'unit, R, R::Offset>
where
    R: gimli::Reader,
{
    type ItemsExtra = DIEItemsExtra<'unit, R>;

    fn parse_items(
        &self,
        items: &mut ir::ItemsBuilder,
        extra: Self::ItemsExtra,
    ) -> Result<(), traits::Error> {
        let Self::ItemsExtra {
            entry_id,
            unit_id,
            addr_size,
            dwarf_version,
            debug_str,
            debug_types,
            rnglists,
            comp_unit,
        } = extra;

        let item = match item_kind(self, debug_types, comp_unit)? {
            Some(kind @ ir::ItemKind::Subroutine(_)) => {
                let name = item_name(self, debug_str)?
                    .unwrap_or(format!("Subroutine[{}][{}]", unit_id, entry_id));
                let id = ir::Id::entry(unit_id, entry_id);
                DieLocationAttributes::try_from(self)?
                    .entity_size(addr_size, dwarf_version, rnglists)?
                    .map(|size| ir::Item::new(id, name, size as u32, kind))
            }
            _ => None,
        };

        if let Some(item) = item {
            items.add_item(item);
        }

        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(
        &self,
        _items: &mut ir::ItemsBuilder,
        _extra: Self::EdgesExtra,
    ) -> Result<(), traits::Error> {
        // TODO: Add edges representing the call graph.
        Ok(())
    }
}
