use gimli;

use super::FallilbleOption;

/// Calculate the item's name. For more information about this, refer to Section 2.15 of
/// the DWARF v5 specification: 'Identifier Names'. Any DIE associated representing an
/// entity that has been given a name may have a `DW_AT_name` attribute. If there was
/// not a name assigned to the entity in the source code, the attribute may either not
/// exist, or be a single null byte.
///
/// If no name was assigned, a name will be decided elsewhere using the
/// ir::ItemKind variant that was determined for the entity.
pub fn item_name<R>(
    die: &gimli::DebuggingInformationEntry<R, R::Offset>,
    debug_str: &gimli::DebugStr<R>,
) -> FallilbleOption<String>
where
    R: gimli::Reader,
{
    match die
        .attr(gimli::DW_AT_name)?
        .and_then(|attr| attr.string_value(&debug_str))
    {
        Some(s) => {
            let name = Some(
                s
                    .to_string()? // This `to_string()` creates a `Result<Cow<'_, str>, _>`.
                    .to_string(), // This `to_string()` creates the String we return.
            );
            Ok(name)
        }
        None => Ok(None),
    }
}
