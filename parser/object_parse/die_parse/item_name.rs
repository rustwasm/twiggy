use gimli;
use twiggy_traits as traits;

use super::FallilbleOption;

/// Calculate the item's name. If no name was assigned, a name will be
/// decided elsewhere using the `ir::ItemKind` variant that was determined
/// for the entity.
///
/// For more information on identifier names and linkage names, refer to
/// Section 2.15 and Section 2.22 of the DWARF v5 specification, respectively.
///
/// > _"Any debugging information entry representing a program entity that
/// > has been given a name may have a DW_AT_name attribute, whose value
/// > is a string representing the name as it appears in the source program."_
///
/// - DWARF v5 Spec. Section 2.15
///
/// > _"A debugging information entry may have a DW_AT_linkage_name attribute
/// > whose value is a null-terminated string describing the object file
/// > linkage name associated with the corresponding entity."_
///
/// -- DWARF v5 Spec. Section 2.22
pub fn item_name<R>(
    die: &gimli::DebuggingInformationEntry<R, R::Offset>,
    dwarf: &gimli::Dwarf<R>,
    unit: &gimli::Unit<R>,
) -> FallilbleOption<String>
where
    R: gimli::Reader,
{
    die.attr_value(gimli::DW_AT_linkage_name)
        .transpose()
        .or_else(|| die.attr_value(gimli::DW_AT_name).transpose())
        .transpose()?
        .map(
            |attr: gimli::read::AttributeValue<R>| -> Result<String, traits::Error> {
                Ok(
                    dwarf
                        .attr_string(unit, attr)?
                        .to_string()? // This `to_string()` creates a `Result<Cow<'_, str>, _>`.
                        .to_string(), // This `to_string()` creates the String we return.
                )
            },
        )
        .transpose()
}
