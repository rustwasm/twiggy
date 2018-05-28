use std::borrow::{Borrow, Cow};

use fallible_iterator::FallibleIterator;
use gimli;
use ir;
use object::{self, Object};
use traits;
use typed_arena::Arena;

use super::Parse;

mod compilation_unit_parse;
mod die_parse;

use self::compilation_unit_parse::{CompUnitEdgesExtra, CompUnitItemsExtra};

// Helper function used to load a given section of the file.
fn load_section<'a, 'file, 'input, Sect, Endian>(
    arena: &'a Arena<Cow<'file, [u8]>>,
    file: &'file object::File<'input>,
    endian: Endian,
) -> Sect
where
    Sect: gimli::Section<gimli::EndianSlice<'a, Endian>>,
    Endian: gimli::Endianity,
    'file: 'input,
    'a: 'file,
{
    let data = file
        .section_data_by_name(Sect::section_name())
        .unwrap_or(Cow::Borrowed(&[]));
    let data_ref = (*arena.alloc(data)).borrow();
    Sect::from(gimli::EndianSlice::new(data_ref, endian))
}

impl<'input> Parse<'input> for object::File<'input> {
    type ItemsExtra = ();

    fn parse_items(
        &self,
        items: &mut ir::ItemsBuilder,
        _extra: Self::ItemsExtra,
    ) -> Result<(), traits::Error> {
        // Identify the file's endianty and create a typed arena to load sections.
        let arena = Arena::new();
        let endian = if self.is_little_endian() {
            gimli::RunTimeEndian::Little
        } else {
            gimli::RunTimeEndian::Big
        };

        // Load the sections of the file containing debugging information.
        let debug_abbrev: gimli::DebugAbbrev<_> = load_section(&arena, self, endian);
        let _debug_aranges: gimli::DebugAranges<_> = load_section(&arena, self, endian);
        let debug_ranges: gimli::DebugRanges<_> = load_section(&arena, self, endian);
        let debug_rnglists: gimli::DebugRngLists<_> = load_section(&arena, self, endian);
        let debug_str: gimli::DebugStr<_> = load_section(&arena, self, endian);
        let debug_types: gimli::DebugTypes<_> = load_section(&arena, self, endian);

        let rnglists = &gimli::RangeLists::new(debug_ranges, debug_rnglists)?;

        // Load the `.debug_info` section, and parse the items in each compilation unit.
        let debug_info: gimli::DebugInfo<_> = load_section(&arena, self, endian);
        let mut compilation_units = debug_info.units().enumerate();
        while let Some((unit_id, unit)) = compilation_units.next()? {
            let extra = CompUnitItemsExtra {
                unit_id,
                debug_abbrev,
                debug_str,
                debug_types,
                rnglists,
            };
            unit.parse_items(items, extra)?
        }

        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(
        &self,
        items: &mut ir::ItemsBuilder,
        _extra: Self::EdgesExtra,
    ) -> Result<(), traits::Error> {
        // Identify the file's endianty and create a typed arena to load sections.
        let arena = Arena::new();
        let endian = if self.is_little_endian() {
            gimli::RunTimeEndian::Little
        } else {
            gimli::RunTimeEndian::Big
        };

        // Load the sections of the file containing debugging information.
        let debug_abbrev: gimli::DebugAbbrev<_> = load_section(&arena, self, endian);

        // Load the `.debug_info` section, and parse the edges in each compilation unit.
        let debug_info: gimli::DebugInfo<_> = load_section(&arena, self, endian);
        let mut compilation_units = debug_info.units().enumerate();
        while let Some((unit_id, unit)) = compilation_units.next()? {
            let extra = CompUnitEdgesExtra {
                unit_id,
                debug_abbrev,
            };
            unit.parse_edges(items, extra)?
        }

        Ok(())
    }
}
