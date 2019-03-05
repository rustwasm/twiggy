//! Parses binaries into `twiggy_ir::Items`.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

#[cfg(feature = "dwarf")]
extern crate fallible_iterator;
#[cfg(feature = "dwarf")]
extern crate gimli;
#[cfg(feature = "dwarf")]
extern crate object;
#[cfg(feature = "dwarf")]
extern crate typed_arena;
extern crate wasmparser;

extern crate twiggy_ir as ir;
extern crate twiggy_traits as traits;

#[cfg(feature = "dwarf")]
mod object_parse;
mod wasm_parse;

use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path;

const WASM_MAGIC_NUMBER: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];

/// Parse the file at the given path into IR items.
pub fn read_and_parse<P: AsRef<path::Path>>(
    path: P,
    mode: traits::ParseMode,
) -> Result<ir::Items, traits::Error> {
    let path = path.as_ref();
    let mut file = fs::File::open(path)?;
    let mut data = vec![];
    file.read_to_end(&mut data)?;

    match mode {
        traits::ParseMode::Wasm => parse_wasm(&data),
        #[cfg(feature = "dwarf")]
        traits::ParseMode::Dwarf => parse_other(&data),
        traits::ParseMode::Auto => match sniff_wasm(path.extension(), &data[..]) {
            true => parse_wasm(&data),
            #[cfg(feature = "dwarf")]
            _ => parse_other(&data),
            #[cfg(not(feature = "dwarf"))]
            _ => parse_fallback(&data),
        },
    }
}

/// Parse the given data into IR items.
pub fn parse(data: &[u8]) -> Result<ir::Items, traits::Error> {
    parse_fallback(data)
}

/// A trait for parsing things into `ir::Item`s.
pub(crate) trait Parse<'a> {
    /// Any extra data needed to parse this type's items.
    type ItemsExtra;

    /// Parse `Self` into one or more `ir::Item`s and add them to the builder.
    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        extra: Self::ItemsExtra,
    ) -> Result<(), traits::Error>;

    /// Any extra data needed to parse this type's edges.
    type EdgesExtra;

    /// Parse edges between items. This is only called *after* we have already
    /// parsed items.
    fn parse_edges(
        &mut self,
        items: &mut ir::ItemsBuilder,
        extra: Self::EdgesExtra,
    ) -> Result<(), traits::Error>;
}

fn sniff_wasm(extension: Option<&OsStr>, data: &[u8]) -> bool {
    match extension.and_then(|s| s.to_str()) {
        Some("wasm") => true,
        _ => data.get(0..4) == Some(&WASM_MAGIC_NUMBER),
    }
}

fn parse_wasm(data: &[u8]) -> Result<ir::Items, traits::Error> {
    let mut items = ir::ItemsBuilder::new(data.len() as u32);

    let mut module1 = wasmparser::ModuleReader::new(data)?;
    module1.parse_items(&mut items, ())?;
    let mut module2 = wasmparser::ModuleReader::new(data)?;
    module2.parse_edges(&mut items, ())?;

    Ok(items.finish())
}

#[cfg(feature = "dwarf")]
fn parse_other(data: &[u8]) -> Result<ir::Items, traits::Error> {
    let mut items = ir::ItemsBuilder::new(data.len() as u32);

    object_parse::parse(&mut items, data)?;

    Ok(items.finish())
}

fn parse_fallback(data: &[u8]) -> Result<ir::Items, traits::Error> {
    parse_wasm(data)
}
