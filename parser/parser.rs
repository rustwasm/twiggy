//! Parses binaries into `svelte_ir::Items`.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

#[macro_use]
extern crate failure;
extern crate parity_wasm as wasm;
extern crate svelte_ir as ir;

use failure::{Fail, ResultExt};
use std::fmt::Write;
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

fn wasm_serialized_size<T>(t: T) -> Result<u32, failure::Error>
where
    T: wasm::elements::Serialize,
    <T as wasm::elements::Serialize>::Error: failure::Fail,
{
    let mut buf = vec![];
    t.serialize(&mut buf)
        .context("could not determine the size of an item")?;
    Ok(buf.len() as u32)
}

fn parse_wasm(data: &[u8]) -> Result<ir::Items, failure::Error> {
    use wasm::elements;

    let mut items = ir::ItemsBuilder::default();

    let module: wasm::elements::Module = elements::deserialize_buffer(data)?;
    // Greedily parse the name section, if it exists.
    let module = match module.parse_names() {
        Ok(m) | Err((_, m)) => m,
    };

    let mut module_name = None;
    let mut function_names = None;
    let mut local_names = None;

    // The custom name sections. Parse these first since they also give us
    // debugging information for later sections.
    for section in module.sections() {
        let name = match *section {
            elements::Section::Name(ref n) => n,
            _ => continue,
        };

        let size = wasm_serialized_size(name.clone())?;

        let name = match *name {
            elements::NameSection::Module(ref m) => {
                module_name = Some(m.name().to_string());
                "module name subsection".to_string()
            }
            elements::NameSection::Function(ref f) => {
                function_names = Some(f.names());
                "function names subsection".to_string()
            }
            elements::NameSection::Local(ref l) => {
                local_names = Some(l.local_names());
                "local names subsection".to_string()
            }
            elements::NameSection::Unparsed { .. } => unreachable!("we pre-parsed names sections"),
        };

        items.add_root(ir::Item::new(name, size, ir::DebugInfo::new()));
    }

    // Custom sections.
    for section in module.sections() {
        let custom = match *section {
            elements::Section::Custom(ref c) => c,
            _ => continue,
        };

        let size = wasm_serialized_size(custom.clone())?;

        let mut name = String::with_capacity("custom section ''".len() + custom.name().len());
        name.push_str("custom section '");
        name.push_str(custom.name());
        name.push_str("'");

        items.add_root(ir::Item::new(name, size, ir::Misc::new()));
    }

    // TODO: start function section
    // TODO: exports section
    //
    // Need these sections sooner rather than later because they let us figure
    // out which items in the graph are roots.

    // TODO: type section
    // TODO: imports section
    // TODO: function section
    // TODO: table section
    // TODO: memory section

    // Global section.
    if let Some(globals) = module.global_section() {
        for (i, g) in globals.entries().iter().enumerate() {
            let mut name = String::with_capacity("global[]".len() + 4);
            write!(&mut name, "global[{}]", i).unwrap();

            let size = wasm_serialized_size(g.clone())?;
            let ty = g.global_type().content_type().to_string();
            items.add_item(ir::Item::new(name, size, ir::Data::new(Some(ty))));
        }
    }

    // TODO: elements section

    // Code section.
    if let Some(code) = module.code_section() {
        let table_offset = module.import_count(elements::ImportCountType::Function);
        for (i, body) in code.bodies().iter().enumerate() {
            let name = function_names
                .as_ref()
                .and_then(|names| names.get((i + table_offset) as u32))
                .map_or_else(
                    || {
                        let mut name = String::with_capacity("code[]".len() + 4);
                        write!(&mut name, "code[{}]", i).unwrap();
                        name
                    },
                    |name| name.to_string(),
                );

            let size = wasm_serialized_size(body.clone())?;
            items.add_item(ir::Item::new(name, size, ir::Code::new()));
        }
    }

    if let Some(data) = module.data_section() {
        for (i, d) in data.entries().iter().enumerate() {
            let mut name = String::with_capacity("data[]".len() + 4);
            write!(&mut name, "data[{}]", i).unwrap();

            let size = wasm_serialized_size(d.clone())?;
            let ty = None;
            items.add_item(ir::Item::new(name, size, ir::Data::new(ty)));
        }
    }

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
