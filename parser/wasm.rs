use super::Parse;
use failure::{self, ResultExt};
use ir;
use parity_wasm::elements;
use std::fmt::Write;

fn serialized_size<T>(t: T) -> Result<u32, failure::Error>
where
    T: elements::Serialize,
    <T as elements::Serialize>::Error: failure::Fail,
{
    let mut buf = vec![];
    t.serialize(&mut buf)
        .context("could not determine the size of an item")?;
    Ok(buf.len() as u32)
}

impl<'a> Parse<'a> for elements::Module {
    type ItemsExtra = ();

    fn parse_items(&self, items: &mut ir::ItemsBuilder, _extra: ()) -> Result<(), failure::Error> {
        let mut function_names = None;

        // The custom name sections. Parse these first since they also give us
        // debugging information for later sections.
        for section in self.sections() {
            let name = match *section {
                elements::Section::Name(ref n) => n,
                _ => continue,
            };
            match *name {
                elements::NameSection::Module(ref m) => {
                    m.parse_items(items, ())?;
                }
                elements::NameSection::Function(ref f) => {
                    function_names = Some(f.names());
                    f.parse_items(items, ())?;
                }
                elements::NameSection::Local(ref l) => {
                    l.parse_items(items, ())?;
                }
                elements::NameSection::Unparsed { .. } => {
                    unreachable!("we pre-parsed names sections")
                }
            };
        }

        for section in self.sections() {
            match *section {
                // Already eagerly parsed above.
                elements::Section::Name(_) => continue,
                elements::Section::Unparsed { .. } => {
                    unreachable!("we eagerly parse all lazily parsed sections (aka names sections)")
                }
                elements::Section::Custom(ref custom) => {
                    custom.parse_items(items, ())?;
                }
                elements::Section::Type(ref ty) => {
                    ty.parse_items(items, ())?;
                }
                elements::Section::Import(ref imports) => {
                    imports.parse_items(items, ())?;
                }
                elements::Section::Function(ref funcs) => {
                    funcs.parse_items(items, ())?;
                }
                elements::Section::Table(ref table) => {
                    table.parse_items(items, ())?;
                }
                elements::Section::Memory(ref mem) => {
                    mem.parse_items(items, ())?;
                }
                elements::Section::Global(ref global) => {
                    global.parse_items(items, ())?;
                }
                elements::Section::Export(ref exports) => {
                    exports.parse_items(items, ())?;
                }
                elements::Section::Start(_) => {
                    let start = StartSection(section);
                    start.parse_items(items, ())?;
                }
                elements::Section::Element(ref elem) => {
                    elem.parse_items(items, ())?;
                }
                elements::Section::Code(ref code) => {
                    code.parse_items(items, (self, function_names))?;
                }
                elements::Section::Data(ref data) => {
                    data.parse_items(items, ())?;
                }
            }
        }

        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, items: &mut ir::ItemsBuilder, _extra: ()) -> Result<(), failure::Error> {
        for section in self.sections() {
            match *section {
                elements::Section::Name(elements::NameSection::Unparsed { .. })
                | elements::Section::Unparsed { .. } => {
                    unreachable!("we eagerly parse all lazily parsed sections")
                }
                elements::Section::Name(elements::NameSection::Module(ref m)) => {
                    m.parse_edges(items, ())?;
                }
                elements::Section::Name(elements::NameSection::Function(ref f)) => {
                    f.parse_edges(items, ())?;
                }
                elements::Section::Name(elements::NameSection::Local(ref l)) => {
                    l.parse_edges(items, ())?;
                }
                elements::Section::Custom(ref custom) => {
                    custom.parse_edges(items, ())?;
                }
                elements::Section::Type(ref ty) => {
                    ty.parse_edges(items, ())?;
                }
                elements::Section::Import(ref imports) => {
                    imports.parse_edges(items, ())?;
                }
                elements::Section::Function(ref funcs) => {
                    funcs.parse_edges(items, self)?;
                }
                elements::Section::Table(ref table) => {
                    table.parse_edges(items, ())?;
                }
                elements::Section::Memory(ref mem) => {
                    mem.parse_edges(items, ())?;
                }
                elements::Section::Global(ref global) => {
                    global.parse_edges(items, ())?;
                }
                elements::Section::Export(ref exports) => {
                    exports.parse_edges(items, self)?;
                }
                elements::Section::Start(_) => {
                    let start = StartSection(section);
                    start.parse_edges(items, self)?;
                }
                elements::Section::Element(ref elem) => {
                    elem.parse_edges(items, self)?;
                }
                elements::Section::Code(ref code) => {
                    code.parse_edges(items, self)?;
                }
                elements::Section::Data(ref data) => {
                    data.parse_edges(items, ())?;
                }
            }
        }
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::ModuleNameSection {
    type ItemsExtra = ();

    fn parse_items(&self, items: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        let name = "\"module name\" subsection";
        let size = serialized_size(self.clone())?;
        items.add_root(self, ir::Item::new(name, size, ir::DebugInfo::new()));
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::FunctionNameSection {
    type ItemsExtra = ();

    fn parse_items(&self, items: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        let name = "\"function names\" subsection";
        let size = serialized_size(self.clone())?;
        items.add_root(self, ir::Item::new(name, size, ir::DebugInfo::new()));
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::LocalNameSection {
    type ItemsExtra = ();

    fn parse_items(&self, items: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        let name = "\"local names\" subsection";
        let size = serialized_size(self.clone())?;
        items.add_root(self, ir::Item::new(name, size, ir::DebugInfo::new()));
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::CustomSection {
    type ItemsExtra = ();

    fn parse_items(&self, items: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        let size = serialized_size(self.clone())?;

        let mut name = String::with_capacity("custom section ''".len() + self.name().len());
        name.push_str("custom section '");
        name.push_str(self.name());
        name.push_str("'");

        items.add_root(self, ir::Item::new(name, size, ir::Misc::new()));
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::TypeSection {
    type ItemsExtra = ();

    fn parse_items(&self, items: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        for (i, ty) in self.types().iter().enumerate() {
            let size = serialized_size(ty.clone())?;
            let mut name = String::with_capacity("type[]".len() + 4);
            write!(&mut name, "type[{}]", i)?;
            items.add_item(ty, ir::Item::new(name, size, ir::Misc::new()));
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::ImportSection {
    type ItemsExtra = ();

    fn parse_items(&self, items: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        for imp in self.entries() {
            let size = serialized_size(imp.clone())?;
            let mut name = String::with_capacity(
                "import ".len() + imp.module().len() + "::".len() + imp.field().len(),
            );
            write!(&mut name, "import {}::{}", imp.module(), imp.field())?;
            items.add_item(imp, ir::Item::new(name, size, ir::Misc::new()));
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::FunctionSection {
    type ItemsExtra = ();

    fn parse_items(&self, items: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        for (i, func) in self.entries().iter().enumerate() {
            // Unfortunately, `Func` does not implement `Serialize`, so we are
            // left with a default size of 1.
            //
            // https://github.com/paritytech/parity-wasm/issues/171
            let size = 1;
            let mut name = String::with_capacity("func[]".len() + 4);
            write!(&mut name, "func[{}]", i)?;
            items.add_item(func, ir::Item::new(name, size, ir::Misc::new()));
        }
        Ok(())
    }

    type EdgesExtra = &'a elements::Module;

    fn parse_edges(
        &self,
        items: &mut ir::ItemsBuilder,
        module: Self::EdgesExtra,
    ) -> Result<(), failure::Error> {
        let types = module.type_section();
        let code = module.code_section();
        for (i, func) in self.entries().iter().enumerate() {
            if let Some(body) = code.and_then(|c| c.bodies().get(i)) {
                items.add_edge(func, body);
            }
            if let Some(ty) = types.and_then(|ts| ts.types().get(func.type_ref() as usize)) {
                items.add_edge(func, ty);
            }
        }
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::TableSection {
    type ItemsExtra = ();

    fn parse_items(&self, items: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        for (i, entry) in self.entries().iter().enumerate() {
            let size = serialized_size(entry.clone())?;
            let mut name = String::with_capacity("table[]".len() + 4);
            write!(&mut name, "table[{}]", i)?;
            items.add_item(entry, ir::Item::new(name, size, ir::Misc::new()));
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::MemorySection {
    type ItemsExtra = ();

    fn parse_items(&self, items: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        for (i, mem) in self.entries().iter().enumerate() {
            let size = serialized_size(mem.clone())?;
            let mut name = String::with_capacity("memory[]".len() + 4);
            write!(&mut name, "memory[{}]", i)?;
            items.add_item(mem, ir::Item::new(name, size, ir::Misc::new()));
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::GlobalSection {
    type ItemsExtra = ();

    fn parse_items(&self, items: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        for (i, g) in self.entries().iter().enumerate() {
            let mut name = String::with_capacity("global[]".len() + 4);
            write!(&mut name, "global[{}]", i).unwrap();

            let size = serialized_size(g.clone())?;
            let ty = g.global_type().content_type().to_string();
            items.add_item(g, ir::Item::new(name, size, ir::Data::new(Some(ty))));
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::ExportSection {
    type ItemsExtra = ();

    fn parse_items(&self, items: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        for exp in self.entries() {
            let mut name = String::with_capacity("export \"\"".len() + exp.field().len());
            write!(&mut name, "export \"{}\"", exp.field())?;
            let size = serialized_size(exp.clone())?;
            items.add_root(exp, ir::Item::new(name, size, ir::Misc::new()));
        }
        Ok(())
    }

    type EdgesExtra = &'a elements::Module;

    fn parse_edges(
        &self,
        items: &mut ir::ItemsBuilder,
        module: Self::EdgesExtra,
    ) -> Result<(), failure::Error> {
        let funcs = module.function_section();
        let tables = module.table_section();
        let memories = module.memory_section();
        let globals = module.global_section();

        let mut func_length = 0 as usize;
        let mut table_length = 0 as usize;
        let mut mem_length = 0 as usize;
        let mut global_length = 0 as usize;

        for exp in self.entries() {
            match *exp.internal() {
                elements::Internal::Function(idx) => {
                    let index = idx as usize - (table_length + mem_length + global_length);
                    if let Some(func) = funcs.and_then(|fs| fs.entries().get(index)) {
                        items.add_edge(exp, func);
                    }
                    func_length += 1;
                }
                elements::Internal::Table(idx) => {
                    let index = idx as usize - (func_length + mem_length + global_length);
                    if let Some(table) = tables.and_then(|ts| ts.entries().get(index)) {
                        items.add_edge(exp, table);
                    }
                    table_length += 1;
                }
                elements::Internal::Memory(idx) => {
                    let index = idx as usize - (func_length + func_length + global_length);
                    if let Some(memory) = memories.and_then(|ms| ms.entries().get(index)) {
                        items.add_edge(exp, memory);
                    }
                    mem_length += 1;
                }
                elements::Internal::Global(idx) => {
                    let index = idx as usize - (func_length + func_length + mem_length);
                    if let Some(global) = globals.and_then(|gs| gs.entries().get(index)) {
                        items.add_edge(exp, global);
                    }
                    global_length += 1;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct StartSection<'a>(&'a elements::Section);

impl<'a> Parse<'a> for StartSection<'a> {
    type ItemsExtra = ();

    fn parse_items(&self, items: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        assert!(match *self.0 {
            elements::Section::Start(_) => true,
            _ => false,
        });

        let size = serialized_size(self.0.clone())?;
        let name = "\"start\" section";
        items.add_root(self.0, ir::Item::new(name, size, ir::Misc::new()));
        Ok(())
    }

    type EdgesExtra = &'a elements::Module;

    fn parse_edges(
        &self,
        items: &mut ir::ItemsBuilder,
        module: Self::EdgesExtra,
    ) -> Result<(), failure::Error> {
        let idx = match *self.0 {
            elements::Section::Start(idx) => idx,
            _ => unreachable!(),
        };

        module
            .function_section()
            .and_then(|fs| fs.entries().get(idx as usize))
            .map(|f| {
                items.add_edge(self.0, f);
            });

        Ok(())
    }
}

impl<'a> Parse<'a> for elements::ElementSection {
    type ItemsExtra = ();

    fn parse_items(&self, items: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        for (i, elem) in self.entries().iter().enumerate() {
            let size = serialized_size(elem.clone())?;
            let mut name = String::with_capacity("elem[]".len() + 4);
            write!(&mut name, "elem[{}]", i)?;
            items.add_item(elem, ir::Item::new(name, size, ir::Misc::new()));
        }
        Ok(())
    }

    type EdgesExtra = &'a elements::Module;

    fn parse_edges(
        &self,
        items: &mut ir::ItemsBuilder,
        module: Self::EdgesExtra,
    ) -> Result<(), failure::Error> {
        let funcs = module.function_section();
        let tables = module.table_section();

        for elem in self.entries() {
            for f in elem.members() {
                funcs.and_then(|fs| fs.entries().get(*f as usize)).map(|f| {
                    items.add_edge(elem, f);
                });
            }
            
            tables
                .and_then(|ts| ts.entries().get(elem.index() as usize))
                .map(|t| {
                    items.add_edge(elem, t);
                });            
        }
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::CodeSection {
    type ItemsExtra = (&'a elements::Module, Option<&'a elements::NameMap>);

    fn parse_items(
        &self,
        items: &mut ir::ItemsBuilder,
        (module, function_names): Self::ItemsExtra,
    ) -> Result<(), failure::Error> {
        let table_offset = module.import_count(elements::ImportCountType::Function);

        for (i, body) in self.bodies().iter().enumerate() {
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

            let size = serialized_size(body.clone())?;
            items.add_item(body, ir::Item::new(name, size, ir::Code::new()));
        }

        Ok(())
    }

    type EdgesExtra = &'a elements::Module;

    fn parse_edges(
        &self,
        items: &mut ir::ItemsBuilder,
        module: Self::EdgesExtra,
    ) -> Result<(), failure::Error> {
        let funcs = module.function_section();
        let globals = module.global_section();

        for body in self.bodies() {
            for op in body.code().elements() {
                match *op {
                    elements::Opcode::Call(idx) => {
                        funcs
                            .and_then(|fs| fs.entries().get(idx as usize))
                            .map(|f| {
                                items.add_edge(body, f);
                            });
                    }

                    // TODO: Rather than looking at indirect calls, need to look
                    // at where the vtables get initialized and/or vtable
                    // indices get pushed onto the stack.
                    //
                    // elements::Opcode::CallIndirect(idx, _reserved) => {}
                    elements::Opcode::GetGlobal(idx) | elements::Opcode::SetGlobal(idx) => {
                        globals
                            .and_then(|gs| gs.entries().get(idx as usize))
                            .map(|g| {
                                items.add_edge(body, g);
                            });
                    }

                    _ => continue,
                }
            }
        }

        Ok(())
    }
}

impl<'a> Parse<'a> for elements::DataSection {
    type ItemsExtra = ();

    fn parse_items(&self, items: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        for (i, d) in self.entries().iter().enumerate() {
            let mut name = String::with_capacity("data[]".len() + 4);
            write!(&mut name, "data[{}]", i).unwrap();

            let size = serialized_size(d.clone())?;
            let ty = None;
            items.add_item(d, ir::Item::new(name, size, ir::Data::new(ty)));
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}
