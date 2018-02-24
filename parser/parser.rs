//! Parses binaries into `svelte_ir::Items`.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

#[macro_use]
extern crate failure;
extern crate parity_wasm;
extern crate svelte_ir as ir;

mod wasm;

use failure::{Fail, ResultExt};
use parity_wasm::elements;
use std::fs;
use std::io::Read;
use std::path;

/// Parse the file at the given path into IR items.
pub fn parse<P: AsRef<path::Path>>(path: P) -> Result<ir::Items, failure::Error> {
    let path = path.as_ref();
    let mut file = fs::File::open(path).context("opening input file")?;
    let mut data = vec![];
    file.read_to_end(&mut data).context("reading input file")?;

    match path.extension().and_then(|s| s.to_str()) {
        Some("wasm") => if let Ok(items) = parse_wasm(&data) {
            return Ok(items);
        },
        _ => {}
    }

    parse_fallback(path, &data)
}

/// A trait for parsing things into `ir::Item`s.
pub(crate) trait Parse<'a> {
    /// Any extra data needed to parse this type's items.
    type ItemsExtra;

    /// Parse `Self` into one or more `ir::Item`s and add them to the builder.
    fn parse_items(
        &self,
        items: &mut ir::ItemsBuilder,
        extra: Self::ItemsExtra,
    ) -> Result<(), failure::Error>;

    /// Any extra data needed to parse this type's edges.
    type EdgesExtra;

    /// Parse edges between items. This is only called *after* we have already
    /// parsed items.
    fn parse_edges(
        &self,
        items: &mut ir::ItemsBuilder,
        extra: Self::EdgesExtra,
    ) -> Result<(), failure::Error>;
}

fn parse_wasm(data: &[u8]) -> Result<ir::Items, failure::Error> {
    let mut items = ir::ItemsBuilder::new(data.len() as u32);

    let module: elements::Module = elements::deserialize_buffer(data)?;

    // Greedily parse the name section, if it exists.
    let module = match module.parse_names() {
        Ok(m) | Err((_, m)) => m,
    };

    module.parse_items(&mut items, ())?;
    module.parse_edges(&mut items, ())?;

    Ok(items.finish())
}

fn parse_fallback(path: &path::Path, data: &[u8]) -> Result<ir::Items, failure::Error> {
    parse_wasm(data)
        .context("could not parse as wasm")
        // This is how we would chain multiple parse failures together:
        //
        // .or_else(|e| {
        //     parse_elf(data)
        //         .context(e)
        //         .context("could not parse as ELF")
        // })
        .map_err(|e| {
            e.context(format_err!("could not parse {}", path.display())).into()
        })
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
