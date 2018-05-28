use fallible_iterator::FallibleIterator;

use gimli;
use traits;

use super::FallilbleOption;

/// This struct holds the values for DWARF attributes related to an object's
/// location in a binary. This is intended to help consolidate the error
/// checking involved in reading attributes, and simplify the process of
/// size calculations for the entity that a debugging information entry (DIE)
/// describes.
///
/// For more information about these attributes, refer to Chapter 2.17 'Code
/// Addresses, Ranges, and Base Addresses' (pg. 51) in the DWARF5 specification.
pub struct DieLocationAttributes<R: gimli::Reader> {
    dw_at_low_pc: Option<gimli::AttributeValue<R>>,
    dw_at_high_pc: Option<gimli::AttributeValue<R>>,
    dw_at_ranges: Option<gimli::AttributeValue<R>>,
}

impl<R: gimli::Reader> DieLocationAttributes<R> {
    /// Try to create a new location attributes instance using the given
    /// debugging information entry (DIE). Reading these attributes may fail,
    /// so this will return a Result rather than a plain `Self`.
    /// TODO: Use the TryFrom trait once it is stable.
    pub fn try_from(
        die: &gimli::DebuggingInformationEntry<R, R::Offset>,
    ) -> Result<Self, traits::Error> {
        Ok(Self {
            dw_at_low_pc: die.attr_value(gimli::DW_AT_low_pc)?,
            dw_at_high_pc: die.attr_value(gimli::DW_AT_high_pc)?,
            dw_at_ranges: die.attr_value(gimli::DW_AT_ranges)?,
        })
    }

    /// Compute the size of a subprogram described by this DIE.
    pub fn entity_size(
        &self,
        addr_size: u8,
        version: u16,
        rnglists: &gimli::RangeLists<R>,
    ) -> FallilbleOption<u64> {
        if let Some(size) = self.contiguous_entity_size()? {
            Ok(Some(size))
        } else if let Some(size) = self.noncontiguous_entity_size(addr_size, version, rnglists)? {
            Ok(Some(size))
        } else {
            Ok(None)
        }
    }

    /// Compute the size of an entity occupying a contiguous range of machine
    /// code addresses in the binary.
    fn contiguous_entity_size(&self) -> FallilbleOption<u64> {
        let dw_at_low_pc: Option<u64> = self.dw_at_low_pc()?;
        match (dw_at_low_pc, &self.dw_at_high_pc) {
            // If DW_AT_high_pc is encoded as an address, return the difference
            // between that value and the DW_AT_low_pc address.
            (Some(low_pc), Some(gimli::AttributeValue::Addr(high_pc))) => {
                Ok(Some(high_pc - low_pc))
            }
            // DWARF 4 allows the DW_AT_high_pc to be encoded as an offset from the
            // address in DW_AT_low_pc. If so, return the offset as the contiguous size.
            (Some(_), Some(gimli::AttributeValue::Udata(offset))) => Ok(Some(*offset)),
            // Return an error if DW_AT_high_pc is not encoded as expected.
            (Some(_), Some(_)) => Err(traits::Error::with_msg(
                "Unexpected DW_AT_high_pc attribute value",
            )),
            // If none of the above conditions were met, this is either a
            // noncontiguous entity, or the DIE does not represent a defintion.
            _ => Ok(None),
        }
    }

    /// Compute the size of an entity occupying a series of non-contigous
    /// ranges of machine code addresses in the binary.
    fn noncontiguous_entity_size(
        &self,
        addr_size: u8,
        version: u16,
        rnglists: &gimli::RangeLists<R>,
    ) -> FallilbleOption<u64> {
        // Identify the base address, which is in the `DW_AT_low_pc` attribute,
        // or the `DW_AT_entry_pc` attribute for noncontiguous entities.
        let base_addr: u64 = if let Some(addr) = self.dw_at_low_pc()? {
            addr
        } else {
            // If neither exists, this DIE does not represent a definition.
            return Ok(None);
        };

        if let Some(offset) = self.dw_at_ranges()? {
            let ranges = rnglists.ranges(offset, version, addr_size, base_addr)?;
            let size = ranges
                .map(|r| r.end - r.begin)
                .fold(0, |res, size| res + size)?;
            Ok(Some(size))
        } else {
            Ok(None)
        }
    }

    /// Return the DW_AT_low_pc attribute as a u64 value representing an address.
    fn dw_at_low_pc(&self) -> FallilbleOption<u64> {
        match &self.dw_at_low_pc {
            Some(gimli::AttributeValue::Addr(address)) => Ok(Some(*address)),
            Some(_) => Err(traits::Error::with_msg(
                "Unexpected base address attribute value",
            )),
            None => Ok(None),
        }
    }

    /// Return the DW_AT_ranges attribute as a u64 value representing an offset
    /// into the `.debug_ranges` section of the file.
    fn dw_at_ranges(
        &self,
    ) -> FallilbleOption<gimli::RangeListsOffset<<R as gimli::Reader>::Offset>> {
        match &self.dw_at_ranges {
            Some(gimli::AttributeValue::RangeListsRef(offset)) => Ok(Some(*offset)),
            Some(_) => Err(traits::Error::with_msg("Unexpected DW_AT_ranges value")),
            None => Ok(None),
        }
    }
}
