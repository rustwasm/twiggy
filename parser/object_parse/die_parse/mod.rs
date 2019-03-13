use gimli;
use twiggy_ir as ir;
use twiggy_traits as traits;

mod item_name;
mod location_attrs;

use self::item_name::item_name;
use self::location_attrs::DieLocationAttributes;
use super::Parse;

/// This type alias is used to represent an option return value for
/// a procedure that could return an Error.
type FallilbleOption<T> = Result<Option<T>, traits::Error>;

/// This struct represents the extra items required by the Parse trait's
/// `parse_items` method. This is constructed by the compilation unit's
/// own implementation of `parse_items`.
pub struct DieItemsExtra<'unit, R>
where
    R: 'unit + gimli::Reader,
{
    pub entry_id: usize,
    pub unit_id: usize,
    pub dwarf: &'unit gimli::Dwarf<R>,
    pub unit: &'unit gimli::Unit<R>,
}

impl<'abbrev, 'unit, R> Parse<'unit>
    for &'_ gimli::DebuggingInformationEntry<'abbrev, 'unit, R, R::Offset>
where
    R: gimli::Reader,
{
    type ItemsExtra = DieItemsExtra<'unit, R>;

    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        extra: Self::ItemsExtra,
    ) -> Result<(), traits::Error> {
        let Self::ItemsExtra {
            entry_id,
            unit_id,
            dwarf,
            unit,
        } = extra;

        let item: ir::Item = match self.tag() {
            gimli::DW_TAG_subprogram => {
                if let Some(size) =
                    DieLocationAttributes::try_from(self)?.entity_size(dwarf, unit)?
                {
                    let id = ir::Id::entry(unit_id, entry_id);
                    let name = item_name(self, dwarf, unit)?;
                    let decorater = format!("Subroutine[{}][{}]", unit_id, entry_id);
                    let kind = ir::Code::new(name, decorater);
                    ir::Item::new(id, size as u32, kind)
                } else {
                    return Ok(());
                }
            }
            _ => return Ok(()),
        };

        items.add_item(item);
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(
        &mut self,
        _items: &mut ir::ItemsBuilder,
        _extra: Self::EdgesExtra,
    ) -> Result<(), traits::Error> {
        // TODO: Add edges representing the call graph.
        Ok(())
    }
}
