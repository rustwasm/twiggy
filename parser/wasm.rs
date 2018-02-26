use super::Parse;
use failure::{self, ResultExt};
use ir::{self, Id};
use parity_wasm::elements::{self, Section};
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
        for (idx, section) in self.sections().iter().enumerate() {
            let name = match *section {
                Section::Name(ref n) => n,
                _ => continue,
            };
            match *name {
                elements::NameSection::Module(ref m) => {
                    m.parse_items(items, idx)?;
                }
                elements::NameSection::Function(ref f) => {
                    function_names = Some(f.names());
                    f.parse_items(items, idx)?;
                }
                elements::NameSection::Local(ref l) => {
                    l.parse_items(items, idx)?;
                }
                elements::NameSection::Unparsed { .. } => {
                    unreachable!("we pre-parsed names sections")
                }
            };
        }

        for (idx, section) in self.sections().iter().enumerate() {
            match *section {
                // Already eagerly parsed above.
                Section::Name(_) => continue,
                Section::Unparsed { .. } => {
                    unreachable!("we eagerly parse all lazily parsed sections (aka names sections)")
                }
                Section::Custom(ref custom) => {
                    custom.parse_items(items, idx)?;
                }
                Section::Type(ref ty) => {
                    ty.parse_items(items, idx)?;
                }
                Section::Import(ref imports) => {
                    imports.parse_items(items, idx)?;
                }
                Section::Function(ref funcs) => {
                    funcs.parse_items(items, idx)?;
                }
                Section::Table(ref table) => {
                    table.parse_items(items, idx)?;
                }
                Section::Memory(ref mem) => {
                    mem.parse_items(items, idx)?;
                }
                Section::Global(ref global) => {
                    global.parse_items(items, idx)?;
                }
                Section::Export(ref exports) => {
                    exports.parse_items(items, idx)?;
                }
                Section::Start(_) => {
                    let start = StartSection(section);
                    start.parse_items(items, idx)?;
                }
                Section::Element(ref elem) => {
                    elem.parse_items(items, idx)?;
                }
                Section::Code(ref code) => {
                    code.parse_items(items, (self, function_names, idx))?;
                }
                Section::Data(ref data) => {
                    data.parse_items(items, idx)?;
                }
            }
        }

        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, items: &mut ir::ItemsBuilder, _extra: ()) -> Result<(), failure::Error> {
        for (idx, section) in self.sections().iter().enumerate() {
            match *section {
                Section::Name(elements::NameSection::Unparsed { .. })
                | Section::Unparsed { .. } => {
                    unreachable!("we eagerly parse all lazily parsed sections")
                }
                Section::Name(elements::NameSection::Module(ref m)) => {
                    m.parse_edges(items, ())?;
                }
                Section::Name(elements::NameSection::Function(ref f)) => {
                    f.parse_edges(items, ())?;
                }
                Section::Name(elements::NameSection::Local(ref l)) => {
                    l.parse_edges(items, ())?;
                }
                Section::Custom(ref custom) => {
                    custom.parse_edges(items, ())?;
                }
                Section::Type(ref ty) => {
                    ty.parse_edges(items, ())?;
                }
                Section::Import(ref imports) => {
                    imports.parse_edges(items, ())?;
                }
                Section::Function(ref funcs) => {
                    funcs.parse_edges(items, (self, idx))?;
                }
                Section::Table(ref table) => {
                    table.parse_edges(items, ())?;
                }
                Section::Memory(ref mem) => {
                    mem.parse_edges(items, ())?;
                }
                Section::Global(ref global) => {
                    global.parse_edges(items, ())?;
                }
                Section::Export(ref exports) => {
                    exports.parse_edges(items, (self, idx))?;
                }
                Section::Start(_) => {
                    let start = StartSection(section);
                    start.parse_edges(items, (self, idx))?;
                }
                Section::Element(ref elem) => {
                    elem.parse_edges(items, (self, idx))?;
                }
                Section::Code(ref code) => {
                    code.parse_edges(items, (self, idx))?;
                }
                Section::Data(ref data) => {
                    data.parse_edges(items, ())?;
                }
            }
        }
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::ModuleNameSection {
    type ItemsExtra = usize;

    fn parse_items(&self, items: &mut ir::ItemsBuilder, idx: usize) -> Result<(), failure::Error> {
        let id = Id::section(idx);
        let name = "\"module name\" subsection";
        let size = serialized_size(self.clone())?;
        items.add_root(ir::Item::new(id, name, size, ir::DebugInfo::new()));
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::FunctionNameSection {
    type ItemsExtra = usize;

    fn parse_items(&self, items: &mut ir::ItemsBuilder, idx: usize) -> Result<(), failure::Error> {
        let id = Id::section(idx);
        let name = "\"function names\" subsection";
        let size = serialized_size(self.clone())?;
        items.add_root(ir::Item::new(id, name, size, ir::DebugInfo::new()));
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::LocalNameSection {
    type ItemsExtra = usize;

    fn parse_items(&self, items: &mut ir::ItemsBuilder, idx: usize) -> Result<(), failure::Error> {
        let id = Id::section(idx);
        let name = "\"local names\" subsection";
        let size = serialized_size(self.clone())?;
        items.add_root(ir::Item::new(id, name, size, ir::DebugInfo::new()));
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::CustomSection {
    type ItemsExtra = usize;

    fn parse_items(&self, items: &mut ir::ItemsBuilder, idx: usize) -> Result<(), failure::Error> {
        let id = Id::section(idx);
        let size = serialized_size(self.clone())?;

        let mut name = String::with_capacity("custom section ''".len() + self.name().len());
        name.push_str("custom section '");
        name.push_str(self.name());
        name.push_str("'");

        items.add_root(ir::Item::new(id, name, size, ir::Misc::new()));
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::TypeSection {
    type ItemsExtra = usize;

    fn parse_items(&self, items: &mut ir::ItemsBuilder, idx: usize) -> Result<(), failure::Error> {
        for (i, ty) in self.types().iter().enumerate() {
            let id = Id::entry(idx, i);
            let size = serialized_size(ty.clone())?;
            let mut name = String::with_capacity("type[]".len() + 4);
            write!(&mut name, "type[{}]", i)?;
            items.add_item(ir::Item::new(id, name, size, ir::Misc::new()));
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::ImportSection {
    type ItemsExtra = usize;

    fn parse_items(&self, items: &mut ir::ItemsBuilder, idx: usize) -> Result<(), failure::Error> {
        for (i, imp) in self.entries().iter().enumerate() {
            let id = Id::entry(idx, i);
            let size = serialized_size(imp.clone())?;
            let mut name = String::with_capacity(
                "import ".len() + imp.module().len() + "::".len() + imp.field().len(),
            );
            write!(&mut name, "import {}::{}", imp.module(), imp.field())?;
            items.add_item(ir::Item::new(id, name, size, ir::Misc::new()));
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::FunctionSection {
    type ItemsExtra = usize;

    fn parse_items(&self, items: &mut ir::ItemsBuilder, idx: usize) -> Result<(), failure::Error> {
        for (i, _func) in self.entries().iter().enumerate() {
            // Unfortunately, `Func` does not implement `Serialize`, so we are
            // left with a default size of 4.
            //
            // https://github.com/paritytech/parity-wasm/issues/171
            let id = Id::entry(idx, i);
            let size = 4;
            let mut name = String::with_capacity("func[]".len() + 4);
            write!(&mut name, "func[{}]", i)?;
            items.add_item(ir::Item::new(id, name, size, ir::Misc::new()));
        }
        Ok(())
    }

    type EdgesExtra = (&'a elements::Module, usize);

    fn parse_edges(
        &self,
        items: &mut ir::ItemsBuilder,
        (module, idx): Self::EdgesExtra,
    ) -> Result<(), failure::Error> {
        let mut type_section = None;
        let mut code_section = None;
        // process type section
        for (mod_id, s) in module.sections().iter().enumerate() {
            match *s {
                Section::Type(_) => type_section = Some(mod_id),
                Section::Code(_) => code_section = Some(mod_id),
                _ => {}
            }
        }
        for (func_i, func) in self.entries().iter().enumerate() {
            let func_id = Id::entry(idx, func_i);

            if let Some(type_idx) = type_section {
                let type_id = Id::entry(type_idx, func.type_ref() as usize);
                items.add_edge(func_id, type_id);
            }
            if let Some(code_idx) = code_section {
                let body_id = Id::entry(code_idx, func_i);
                items.add_edge(func_id, body_id);
            }
        }

        Ok(())
    }
}

impl<'a> Parse<'a> for elements::TableSection {
    type ItemsExtra = usize;

    fn parse_items(&self, items: &mut ir::ItemsBuilder, idx: usize) -> Result<(), failure::Error> {
        for (i, entry) in self.entries().iter().enumerate() {
            let id = Id::entry(idx, i);
            let size = serialized_size(entry.clone())?;
            let mut name = String::with_capacity("table[]".len() + 4);
            write!(&mut name, "table[{}]", i)?;
            items.add_item(ir::Item::new(id, name, size, ir::Misc::new()));
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::MemorySection {
    type ItemsExtra = usize;

    fn parse_items(&self, items: &mut ir::ItemsBuilder, idx: usize) -> Result<(), failure::Error> {
        for (i, mem) in self.entries().iter().enumerate() {
            let id = Id::entry(idx, i);
            let size = serialized_size(mem.clone())?;
            let mut name = String::with_capacity("memory[]".len() + 4);
            write!(&mut name, "memory[{}]", i)?;
            items.add_item(ir::Item::new(id, name, size, ir::Misc::new()));
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::GlobalSection {
    type ItemsExtra = usize;

    fn parse_items(&self, items: &mut ir::ItemsBuilder, idx: usize) -> Result<(), failure::Error> {
        for (i, g) in self.entries().iter().enumerate() {
            let id = Id::entry(idx, i);
            let mut name = String::with_capacity("global[]".len() + 4);
            write!(&mut name, "global[{}]", i).unwrap();

            let size = serialized_size(g.clone())?;
            let ty = g.global_type().content_type().to_string();
            items.add_item(ir::Item::new(id, name, size, ir::Data::new(Some(ty))));
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::ExportSection {
    type ItemsExtra = usize;

    fn parse_items(&self, items: &mut ir::ItemsBuilder, idx: usize) -> Result<(), failure::Error> {
        for (i, exp) in self.entries().iter().enumerate() {
            let id = Id::entry(idx, i);
            let mut name = String::with_capacity("export \"\"".len() + exp.field().len());
            write!(&mut name, "export \"{}\"", exp.field())?;
            let size = serialized_size(exp.clone())?;
            items.add_root(ir::Item::new(id, name, size, ir::Misc::new()));
        }
        Ok(())
    }

    type EdgesExtra = (&'a elements::Module, usize);

    fn parse_edges(
        &self,
        items: &mut ir::ItemsBuilder,
        (module, idx): Self::EdgesExtra,
    ) -> Result<(), failure::Error> {
        let mut func_section = None;
        let mut table_section = None;
        let mut memory_section = None;
        let mut global_section = None;

        for (mod_id, s) in module.sections().iter().enumerate() {
            match *s {
                Section::Function(_) => func_section = Some(mod_id),
                Section::Table(_) => table_section = Some(mod_id),
                Section::Memory(_) => memory_section = Some(mod_id),
                Section::Global(_) => global_section = Some(mod_id),
                _ => {}
            }
        }

        for (i, exp) in self.entries().iter().enumerate() {
            let exp_id = Id::entry(idx, i);
            match *exp.internal() {
                elements::Internal::Function(f_i) => {
                    if let Some(func_idx) = func_section {
                        items.add_edge(exp_id, Id::entry(func_idx, f_i as usize));
                    }
                }
                elements::Internal::Table(f_i) => {
                    if let Some(table_idx) = table_section {
                        items.add_edge(exp_id, Id::entry(table_idx, f_i as usize));
                    }
                }
                elements::Internal::Memory(f_i) => {
                    if let Some(memory_idx) = memory_section {
                        items.add_edge(exp_id, Id::entry(memory_idx, f_i as usize));
                    }
                }
                elements::Internal::Global(f_i) => {
                    if let Some(global_idx) = global_section {
                        items.add_edge(exp_id, Id::entry(global_idx, f_i as usize));
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct StartSection<'a>(&'a Section);

impl<'a> Parse<'a> for StartSection<'a> {
    type ItemsExtra = usize;

    fn parse_items(&self, items: &mut ir::ItemsBuilder, idx: usize) -> Result<(), failure::Error> {
        assert!(match *self.0 {
            Section::Start(_) => true,
            _ => false,
        });

        let id = Id::section(idx);
        let size = serialized_size(self.0.clone())?;
        let name = "\"start\" section";
        items.add_root(ir::Item::new(id, name, size, ir::Misc::new()));
        Ok(())
    }

    type EdgesExtra = (&'a elements::Module, usize);

    fn parse_edges(
        &self,
        items: &mut ir::ItemsBuilder,
        (module, idx): Self::EdgesExtra,
    ) -> Result<(), failure::Error> {
        let f_i = match *self.0 {
            Section::Start(i) => i,
            _ => unreachable!(),
        };

        let mut func_section = None;
        for (mod_id, s) in module.sections().iter().enumerate() {
            match *s {
                Section::Function(_) => func_section = Some(mod_id),
                _ => {}
            }
        }
        if let Some(func_idx) = func_section {
            items.add_edge(Id::section(idx), Id::entry(func_idx, f_i as usize));
        }

        Ok(())
    }
}

impl<'a> Parse<'a> for elements::ElementSection {
    type ItemsExtra = usize;

    fn parse_items(&self, items: &mut ir::ItemsBuilder, idx: usize) -> Result<(), failure::Error> {
        for (i, elem) in self.entries().iter().enumerate() {
            let id = Id::entry(idx, i);
            let size = serialized_size(elem.clone())?;
            let mut name = String::with_capacity("elem[]".len() + 4);
            write!(&mut name, "elem[{}]", i)?;
            items.add_item(ir::Item::new(id, name, size, ir::Misc::new()));
        }
        Ok(())
    }

    type EdgesExtra = (&'a elements::Module, usize);

    fn parse_edges(
        &self,
        items: &mut ir::ItemsBuilder,
        (module, idx): Self::EdgesExtra,
    ) -> Result<(), failure::Error> {
        let mut func_section = None;
        let mut table_section = None;

        for (mod_id, s) in module.sections().iter().enumerate() {
            match *s {
                Section::Function(_) => func_section = Some(mod_id),
                Section::Table(_) => table_section = Some(mod_id),
                _ => {}
            }
        }

        let num_imported_funcs = module.import_count(elements::ImportCountType::Function);
        for (i, elem) in self.entries().iter().enumerate() {
            let elem_id = Id::entry(idx, i);
            if let Some(table_idx) = table_section {
                let entry_id = Id::entry(table_idx, elem.index() as usize);
                items.add_edge(elem_id, entry_id);
            }
            if let Some(func_idx) = func_section {
                for &f_i in elem.members() {
                    let f_id = Id::entry(func_idx, f_i as usize - num_imported_funcs);
                    items.add_edge(elem_id, f_id);
                }
            }
        }
        Ok(())
    }
}

impl<'a> Parse<'a> for elements::CodeSection {
    type ItemsExtra = (&'a elements::Module, Option<&'a elements::NameMap>, usize);

    fn parse_items(
        &self,
        items: &mut ir::ItemsBuilder,
        (module, function_names, idx): Self::ItemsExtra,
    ) -> Result<(), failure::Error> {
        let table_offset = module.import_count(elements::ImportCountType::Function);

        for (i, body) in self.bodies().iter().enumerate() {
            let id = Id::entry(idx, i);
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
            items.add_item(ir::Item::new(id, name, size, ir::Code::new()));
        }

        Ok(())
    }

    type EdgesExtra = (&'a elements::Module, usize);

    fn parse_edges(
        &self,
        items: &mut ir::ItemsBuilder,
        (module, idx): Self::EdgesExtra,
    ) -> Result<(), failure::Error> {
        let mut func_section = None;
        let mut global_section = None;

        for (mod_id, s) in module.sections().iter().enumerate() {
            match *s {
                Section::Function(_) => func_section = Some(mod_id),
                Section::Global(_) => global_section = Some(mod_id),
                _ => {}
            }
        }

        for (b_i, body) in self.bodies().iter().enumerate() {
            use parity_wasm::elements::Opcode::*;

            let body_id = Id::entry(idx, b_i);
            let code = body.code().elements();

            for i in 0..code.len() {
                match code[i] {
                    Call(idx) => {
                        if let Some(func_idx) = func_section {
                            let f_id = Id::entry(func_idx as usize, idx as usize);
                            items.add_edge(body_id, f_id);
                        }
                    }

                    // TODO: Rather than looking at indirect calls, need to look
                    // at where the vtables get initialized and/or vtable
                    // indices get pushed onto the stack.
                    //
                    // elements::Opcode::CallIndirect(idx, _reserved) => {}
                    GetGlobal(idx) | SetGlobal(idx) => {
                        if let Some(global_idx) = global_section {
                            let g_id = Id::entry(global_idx, idx as usize);
                            items.add_edge(body_id, g_id);
                        }
                    }

                    I32Load(_, off)
                    | I32Load8S(_, off)
                    | I32Load8U(_, off)
                    | I32Load16S(_, off)
                    | I32Load16U(_, off)
                    | I64Load(_, off)
                    | I64Load8S(_, off)
                    | I64Load8U(_, off)
                    | I64Load16S(_, off)
                    | I64Load16U(_, off)
                    | I64Load32S(_, off)
                    | I64Load32U(_, off)
                    | F32Load(_, off)
                    | F64Load(_, off) => {
                        if i > 0 {
                            if let I32Const(base) = code[i - 1] {
                                if let Some(data_id) = items.get_data(base as u32 + off) {
                                    items.add_edge(body_id, data_id);
                                }
                            }
                        }
                    }

                    _ => continue,
                }
            }
        }

        Ok(())
    }
}

impl<'a> Parse<'a> for elements::DataSection {
    type ItemsExtra = usize;

    fn parse_items(&self, items: &mut ir::ItemsBuilder, idx: usize) -> Result<(), failure::Error> {
        for (i, d) in self.entries().iter().enumerate() {
            use parity_wasm::elements::Opcode::*;

            let id = Id::entry(idx, i);
            let mut name = String::with_capacity("data[]".len() + 4);
            write!(&mut name, "data[{}]", i).unwrap();

            let size = serialized_size(d.clone())?; // serialized size
            let length = d.value().len(); // size of data
            let ty = None;
            let offset_code = d.offset().code();
            let offset = offset_code.get(0).and_then(|op| match *op {
                I32Const(o) => Some(o as i64),
                I64Const(o) => Some(o),
                _ => None,
            });

            items.add_item(ir::Item::new(id, name, size, ir::Data::new(ty)));

            if let Some(off) = offset {
                items.link_data(off, length, id);
            }
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), failure::Error> {
        Ok(())
    }
}
