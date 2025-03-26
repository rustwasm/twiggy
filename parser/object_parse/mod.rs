use anyhow::anyhow;
use object::{
    elf, Architecture, File, Object, ObjectSection, ObjectSegment, ObjectSymbol, SectionFlags,
};
use twiggy_ir as ir;

fn maybe_thumb_real_addr(file: &File, addr: u64) -> u64 {
    match file.architecture() {
        Architecture::Arm => {
            addr & !1 // Clear LSB. LSB is set when the function is a Thumb function.
        }
        _ => addr,
    }
}

pub fn parse(data: &[u8]) -> anyhow::Result<ir::Items> {
    let file: File =
        File::parse(data).map_err(|err| anyhow!("Failed to parse data with err: {:?}", err))?;

    let mut alloc_size = 0;
    for segment in file.segments() {
        alloc_size += segment.size();
    }

    let mut items = ir::ItemsBuilder::new(alloc_size as u32);

    for symbol in file.symbols() {
        if !symbol.is_definition() {
            continue;
        }

        if symbol.size() == 0 {
            continue;
        }

        // Filter out symbols in non-allocated sections. Their symbol values do not correspond to
        // actual runtime addresses.
        match file
            .section_by_index(symbol.section_index().unwrap())
            .unwrap()
            .flags()
        {
            SectionFlags::Elf { sh_flags } => {
                if sh_flags as u32 & elf::SHF_ALLOC != elf::SHF_ALLOC {
                    continue;
                }
            }
            _ => {}
        }

        if !file.segments().any(|segment| {
            segment
                .data_range(maybe_thumb_real_addr(&file, symbol.address()), 1)
                .ok()
                .flatten()
                .is_some()
        }) {
            // Symbol not part of any loaded segment
            continue;
        }

        let id = ir::Id::entry(symbol.section_index().unwrap().0, symbol.index().0);
        let name = symbol.name().unwrap();
        let kind: ir::ItemKind = ir::Code::new(name).into();
        let item = ir::Item::new(id, name, symbol.size() as u32, kind);
        if maybe_thumb_real_addr(&file, symbol.address())
            == maybe_thumb_real_addr(&file, file.entry())
        {
            items.add_root(item);
        } else {
            items.add_item(item);
        }
    }

    Ok(items.finish())
}
