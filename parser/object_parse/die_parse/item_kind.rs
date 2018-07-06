#[cfg(feature = "dwarf")]
use gimli;
use ir;

use super::FallilbleOption;

/// Calculate the kind of IR item to represent the code or data associated with
/// a given debugging information entry.
pub fn item_kind<R>(
    die: &gimli::DebuggingInformationEntry<R, R::Offset>,
    _debug_types: &gimli::DebugTypes<R>,
    _compilation_unit: &gimli::CompilationUnitHeader<R, <R as gimli::Reader>::Offset>,
) -> FallilbleOption<ir::ItemKind>
where
    R: gimli::Reader,
{
    let item_kind = match die.tag() {
        gimli::DW_TAG_subprogram => Some(ir::Subroutine::new().into()),
        _ => None,
    };

    Ok(item_kind)
}
