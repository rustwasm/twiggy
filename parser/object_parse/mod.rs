use anyhow::anyhow;
use fallible_iterator::FallibleIterator;
use gimli;
use object::{Object, ObjectSection, ObjectSegment};
use std::borrow::{Borrow, Cow};
use twiggy_ir as ir;
use typed_arena::Arena;

mod compilation_unit_parse;
mod die_parse;

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
        .section_by_name(Sect::section_name())
        .and_then(|section| section.uncompressed_data().ok())
        .unwrap_or(Cow::Borrowed(&[]));
    let data_ref = (*arena.alloc(data)).borrow();
    Sect::from(gimli::EndianSlice::new(data_ref, endian))
}

pub fn parse(data: &[u8]) -> anyhow::Result<ir::Items> {
    let file: object::File = object::File::parse(data)
        .map_err(|err| anyhow!("Failed to parse data with err: {:?}", err))?;

    // Identify the file's endianty and create a typed arena to load sections.
    let arena = Arena::new();
    let endian = if file.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };

    // Load the sections of the file containing debugging information.
    let debug_abbrev: gimli::DebugAbbrev<_> = load_section(&arena, &file, endian);
    let debug_addr: gimli::DebugAddr<_> = load_section(&arena, &file, endian);
    let debug_info: gimli::DebugInfo<_> = load_section(&arena, &file, endian);
    let debug_line: gimli::DebugLine<_> = load_section(&arena, &file, endian);
    let debug_line_str: gimli::DebugLineStr<_> = load_section(&arena, &file, endian);
    let debug_str: gimli::DebugStr<_> = load_section(&arena, &file, endian);
    let debug_str_offsets: gimli::DebugStrOffsets<_> = load_section(&arena, &file, endian);
    let debug_ranges: gimli::DebugRanges<_> = load_section(&arena, &file, endian);
    let debug_rnglists: gimli::DebugRngLists<_> = load_section(&arena, &file, endian);
    let ranges = gimli::RangeLists::new(debug_ranges, debug_rnglists);
    let dwarf = gimli::Dwarf {
        debug_abbrev,
        debug_addr,
        debug_info,
        debug_line,
        debug_line_str,
        debug_str,
        debug_str_offsets,
        ranges,
        ..Default::default()
    };

    let mut alloc_size = 0;
    for segment in file.segments() {
        alloc_size += segment.size();
    }

    let mut items = ir::ItemsBuilder::new(alloc_size as u32);
    parse_items(&mut items, &dwarf)?;
    parse_edges(&mut items, &dwarf)?;
    Ok(items.finish())
}

fn parse_items<R: gimli::Reader>(
    items: &mut ir::ItemsBuilder,
    dwarf: &gimli::Dwarf<R>,
) -> anyhow::Result<()> {
    // Parse the items in each compilation unit.
    let mut headers = dwarf.units().enumerate();
    while let Some((unit_id, header)) = headers.next()? {
        let unit = dwarf.unit(header)?;
        compilation_unit_parse::parse_items(items, dwarf, &unit, unit_id)?
    }

    Ok(())
}

fn parse_edges<R: gimli::Reader>(
    items: &mut ir::ItemsBuilder,
    dwarf: &gimli::Dwarf<R>,
) -> anyhow::Result<()> {
    // Parse the edges in each compilation unit.
    let mut headers = dwarf.units().enumerate();
    while let Some((unit_id, header)) = headers.next()? {
        let unit = dwarf.unit(header)?;
        compilation_unit_parse::parse_edges(items, &unit, unit_id)?
    }

    Ok(())
}
