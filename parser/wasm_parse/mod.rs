use super::Parse;
use std::collections::HashMap;
use twiggy_ir::{self as ir, Id};
use twiggy_traits as traits;
use wasmparser::SectionWithLimitedItems;
use wasmparser::{self, Operator, SectionReader, Type};

#[derive(Default)]
pub struct SectionIndices {
    type_: Option<usize>,
    code: Option<usize>,
    functions: Vec<Id>,
    tables: Vec<Id>,
    memories: Vec<Id>,
    globals: Vec<Id>,
}

impl<'a> Parse<'a> for wasmparser::ModuleReader<'a> {
    type ItemsExtra = ();

    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        _extra: (),
    ) -> Result<(), traits::Error> {
        let initial_offset = self.current_position();
        let mut sections = Vec::new();
        while !self.eof() {
            let start = self.current_position();
            let section = self.read()?;
            let size = self.current_position() - start;
            sections.push((size as u32, section));
        }

        let id = Id::section(sections.len());
        items.add_root(ir::Item::new(
            id,
            initial_offset as u32,
            ir::Misc::new("wasm magic bytes"),
        ));

        // Before we actually parse any items prepare to parse a few sections
        // below, namely the code section. When parsing the code section we want
        // to try to assign human-readable names so we need the name section, if
        // present. Additionally we need to look at the number of imported
        // functions to handle the wasm function index space correctly.
        let mut names = HashMap::new();
        let mut imported_functions = 0;
        for (_, section) in sections.iter() {
            match section.code {
                wasmparser::SectionCode::Custom { name: "name", .. } => {
                    for subsection in section.get_name_section_reader()? {
                        let f = match subsection? {
                            wasmparser::Name::Function(f) => f,
                            _ => continue,
                        };
                        let mut map = f.get_map()?;
                        for _ in 0..map.get_count() {
                            let naming = map.read()?;
                            names.insert(naming.index as usize, naming.name);
                        }
                    }
                }
                wasmparser::SectionCode::Import => {
                    for import in section.get_import_section_reader()? {
                        if let wasmparser::ImportSectionEntryType::Function(_) = import?.ty {
                            imported_functions += 1;
                        }
                    }
                }
                _ => {}
            }
        }

        for (idx, (size, section)) in sections.into_iter().enumerate() {
            let start = items.size_added();
            let name = match section.code {
                wasmparser::SectionCode::Custom { name, .. } => {
                    CustomSectionReader(name, section).parse_items(items, idx)?;
                    format!("custom section '{}' headers", name)
                }
                wasmparser::SectionCode::Type => {
                    section.get_type_section_reader()?.parse_items(items, idx)?;
                    "type section headers".to_string()
                }
                wasmparser::SectionCode::Import => {
                    section
                        .get_import_section_reader()?
                        .parse_items(items, idx)?;
                    "import section headers".to_string()
                }
                wasmparser::SectionCode::Function => {
                    section
                        .get_function_section_reader()?
                        .parse_items(items, (idx, imported_functions, &names))?;
                    "function section headers".to_string()
                }
                wasmparser::SectionCode::Table => {
                    section
                        .get_table_section_reader()?
                        .parse_items(items, idx)?;
                    "table section headers".to_string()
                }
                wasmparser::SectionCode::Memory => {
                    section
                        .get_memory_section_reader()?
                        .parse_items(items, idx)?;
                    "memory section headers".to_string()
                }
                wasmparser::SectionCode::Global => {
                    section
                        .get_global_section_reader()?
                        .parse_items(items, idx)?;
                    "global section headers".to_string()
                }
                wasmparser::SectionCode::Export => {
                    section
                        .get_export_section_reader()?
                        .parse_items(items, idx)?;
                    "export section headers".to_string()
                }
                wasmparser::SectionCode::Start => {
                    StartSection(section).parse_items(items, idx)?;
                    "start section headers".to_string()
                }
                wasmparser::SectionCode::Element => {
                    section
                        .get_element_section_reader()?
                        .parse_items(items, idx)?;
                    "element section headers".to_string()
                }
                wasmparser::SectionCode::Code => {
                    section
                        .get_code_section_reader()?
                        .parse_items(items, (idx, imported_functions, &names))?;
                    "code section headers".to_string()
                }
                wasmparser::SectionCode::Data => {
                    section.get_data_section_reader()?.parse_items(items, idx)?;
                    "data section headers".to_string()
                }
                wasmparser::SectionCode::DataCount => {
                    DataCountSection(section).parse_items(items, idx)?;
                    "data count section headers".to_string()
                }
            };
            let id = Id::section(idx);
            let added = items.size_added() - start;
            assert!(added <= size);
            let item_kind = ir::Misc::new(&name);
            items.add_root(ir::Item::new(id, size - added, item_kind));
        }

        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(
        &mut self,
        items: &mut ir::ItemsBuilder,
        _extra: (),
    ) -> Result<(), traits::Error> {
        let mut sections = Vec::new();
        while !self.eof() {
            sections.push(self.read()?);
        }

        // Like above we do some preprocessing here before actually drawing all
        // the edges below. Here we primarily want to learn some properties of
        // the wasm module, such as what `Id` is mapped to all index spaces in
        // the wasm module. To handle that we build up all this data in
        // `SectionIndices` here as we parse all the various sections.
        let mut indices = SectionIndices::default();
        for (idx, section) in sections.iter().enumerate() {
            match section.code {
                wasmparser::SectionCode::Type => {
                    indices.type_ = Some(idx);
                }
                wasmparser::SectionCode::Code => {
                    indices.code = Some(idx);
                }
                wasmparser::SectionCode::Import => {
                    let reader = section.get_import_section_reader()?;
                    for (i, import) in reader.into_iter().enumerate() {
                        let id = Id::entry(idx, i);
                        match import?.ty {
                            wasmparser::ImportSectionEntryType::Function(_) => {
                                indices.functions.push(id);
                            }
                            wasmparser::ImportSectionEntryType::Table(_) => {
                                indices.tables.push(id);
                            }
                            wasmparser::ImportSectionEntryType::Memory(_) => {
                                indices.memories.push(id);
                            }
                            wasmparser::ImportSectionEntryType::Global(_) => {
                                indices.globals.push(id);
                            }
                        }
                    }
                }
                wasmparser::SectionCode::Global => {
                    for i in 0..section.get_global_section_reader()?.get_count() {
                        let id = Id::entry(idx, i as usize);
                        indices.globals.push(id);
                    }
                }
                wasmparser::SectionCode::Memory => {
                    for i in 0..section.get_memory_section_reader()?.get_count() {
                        let id = Id::entry(idx, i as usize);
                        indices.memories.push(id);
                    }
                }
                wasmparser::SectionCode::Table => {
                    for i in 0..section.get_table_section_reader()?.get_count() {
                        let id = Id::entry(idx, i as usize);
                        indices.tables.push(id);
                    }
                }
                wasmparser::SectionCode::Function => {
                    for i in 0..section.get_function_section_reader()?.get_count() {
                        let id = Id::entry(idx, i as usize);
                        indices.functions.push(id);
                    }
                }
                _ => {}
            }
        }

        for (idx, section) in sections.into_iter().enumerate() {
            match section.code {
                wasmparser::SectionCode::Custom { name, .. } => {
                    CustomSectionReader(name, section).parse_edges(items, ())?;
                }
                wasmparser::SectionCode::Type => {
                    indices.type_ = Some(idx);
                    section.get_type_section_reader()?.parse_edges(items, ())?;
                }
                wasmparser::SectionCode::Import => {
                    section
                        .get_import_section_reader()?
                        .parse_edges(items, ())?;
                }
                wasmparser::SectionCode::Function => {
                    section
                        .get_function_section_reader()?
                        .parse_edges(items, (&indices, idx))?;
                }
                wasmparser::SectionCode::Table => {
                    section.get_table_section_reader()?.parse_edges(items, ())?;
                }
                wasmparser::SectionCode::Memory => {
                    section
                        .get_memory_section_reader()?
                        .parse_edges(items, ())?;
                }
                wasmparser::SectionCode::Global => {
                    section
                        .get_global_section_reader()?
                        .parse_edges(items, ())?;
                }
                wasmparser::SectionCode::Export => {
                    section
                        .get_export_section_reader()?
                        .parse_edges(items, (&indices, idx))?;
                }
                wasmparser::SectionCode::Start => {
                    StartSection(section).parse_edges(items, (&indices, idx))?;
                }
                wasmparser::SectionCode::Element => {
                    section
                        .get_element_section_reader()?
                        .parse_edges(items, (&indices, idx))?;
                }
                wasmparser::SectionCode::Code => {
                    indices.type_ = Some(idx);
                    section
                        .get_code_section_reader()?
                        .parse_edges(items, (&indices, idx))?;
                }
                wasmparser::SectionCode::Data => {
                    section.get_data_section_reader()?.parse_edges(items, ())?;
                }
                wasmparser::SectionCode::DataCount => {
                    DataCountSection(section).parse_edges(items, ())?;
                }
            }
        }

        Ok(())
    }
}

impl<'a> Parse<'a> for wasmparser::NameSectionReader<'a> {
    type ItemsExtra = usize;

    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        idx: usize,
    ) -> Result<(), traits::Error> {
        let mut i = 0;
        while !self.eof() {
            let start = self.original_position();
            let subsection = self.read()?;
            let size = (self.original_position() - start) as u32;
            let name = match subsection {
                wasmparser::Name::Module(_) => "\"module name\" subsection",
                wasmparser::Name::Function(_) => "\"function names\" subsection",
                wasmparser::Name::Local(_) => "\"local names\" subsection",
            };
            let id = Id::entry(idx, i);
            let item_kind = ir::DebugInfo::new(&name);
            items.add_root(ir::Item::new(id, size, item_kind));
            i += 1;
        }

        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&mut self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), traits::Error> {
        Ok(())
    }
}

struct CustomSectionReader<'a>(&'a str, wasmparser::Section<'a>);

impl<'a> Parse<'a> for CustomSectionReader<'a> {
    type ItemsExtra = usize;

    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        idx: usize,
    ) -> Result<(), traits::Error> {
        let name = self.0;
        if name == "name" {
            self.1.get_name_section_reader()?.parse_items(items, idx)?;
        } else {
            let range = self.1.get_binary_reader().range();
            let size = (range.end - range.start) as u32;
            let id = Id::entry(idx, 0);
            let name = format!("custom section '{}'", self.0);
            let item_kind = ir::Misc::new(&name);
            items.add_item(ir::Item::new(id, size, item_kind));
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&mut self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), traits::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for wasmparser::TypeSectionReader<'a> {
    type ItemsExtra = usize;

    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        idx: usize,
    ) -> Result<(), traits::Error> {
        for (i, ty) in iterate_with_size(self).enumerate() {
            let (ty, size) = ty?;
            let id = Id::entry(idx, i);

            let mut name = format!("type[{}]: (", i);
            for (i, param) in ty.params.iter().enumerate() {
                if i != 0 {
                    name.push_str(", ");
                }
                name.push_str(ty2str(*param));
            }
            name.push_str(") -> ");

            match ty.returns.len() {
                0 => name.push_str("nil"),
                1 => name.push_str(ty2str(ty.returns[0])),
                _ => {
                    name.push_str("(");
                    for (i, result) in ty.returns.iter().enumerate() {
                        if i != 0 {
                            name.push_str(", ");
                        }
                        name.push_str(ty2str(*result));
                    }
                    name.push_str(")");
                }
            }

            let item_kind = ir::Misc::new(&name);
            items.add_item(ir::Item::new(id, size, item_kind));
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&mut self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), traits::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for wasmparser::ImportSectionReader<'a> {
    type ItemsExtra = usize;

    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        idx: usize,
    ) -> Result<(), traits::Error> {
        for (i, imp) in iterate_with_size(self).enumerate() {
            let (imp, size) = imp?;
            let id = Id::entry(idx, i);
            let name = format!("import {}::{}", imp.module, imp.field);
            let item_kind = ir::Misc::new(&name);
            items.add_item(ir::Item::new(id, size, item_kind));
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&mut self, _: &mut ir::ItemsBuilder, (): ()) -> Result<(), traits::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for wasmparser::FunctionSectionReader<'a> {
    type ItemsExtra = (usize, usize, &'a HashMap<usize, &'a str>);

    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        (idx, imported_functions, names): Self::ItemsExtra,
    ) -> Result<(), traits::Error> {
        for (i, func) in iterate_with_size(self).enumerate() {
            let (_func, size) = func?;
            let id = Id::entry(idx, i);
            let name = names
                .get(&(i + imported_functions))
                .map(ToString::to_string);
            let decorator = format!("func[{}]", i);
            let item_kind = ir::Function::new(name, decorator);
            items.add_item(ir::Item::new(id, size, item_kind));
        }
        Ok(())
    }

    type EdgesExtra = (&'a SectionIndices, usize);

    fn parse_edges(
        &mut self,
        items: &mut ir::ItemsBuilder,
        (indices, idx): Self::EdgesExtra,
    ) -> Result<(), traits::Error> {
        for (func_i, type_ref) in iterate_with_size(self).enumerate() {
            let (type_ref, _) = type_ref?;
            let func_id = Id::entry(idx, func_i);

            if let Some(type_idx) = indices.type_ {
                let type_id = Id::entry(type_idx, type_ref as usize);
                items.add_edge(func_id, type_id);
            }
            if let Some(code_idx) = indices.code {
                let body_id = Id::entry(code_idx, func_i);
                items.add_edge(func_id, body_id);
            }
        }

        Ok(())
    }
}

impl<'a> Parse<'a> for wasmparser::TableSectionReader<'a> {
    type ItemsExtra = usize;

    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        idx: usize,
    ) -> Result<(), traits::Error> {
        for (i, entry) in iterate_with_size(self).enumerate() {
            let (_entry, size) = entry?;
            let id = Id::entry(idx, i);
            let name = format!("table[{}]", i);
            let item_kind = ir::Misc::new(&name);
            items.add_root(ir::Item::new(id, size, item_kind));
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&mut self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), traits::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for wasmparser::MemorySectionReader<'a> {
    type ItemsExtra = usize;

    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        idx: usize,
    ) -> Result<(), traits::Error> {
        for (i, mem) in iterate_with_size(self).enumerate() {
            let (_mem, size) = mem?;
            let id = Id::entry(idx, i);
            let name = format!("memory[{}]", i);
            let item_kind = ir::Misc::new(&name);
            items.add_item(ir::Item::new(id, size, item_kind));
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&mut self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), traits::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for wasmparser::GlobalSectionReader<'a> {
    type ItemsExtra = usize;

    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        idx: usize,
    ) -> Result<(), traits::Error> {
        for (i, g) in iterate_with_size(self).enumerate() {
            let (g, size) = g?;
            let id = Id::entry(idx, i);
            let name = format!("global[{}]", i);
            let ty = ty2str(g.ty.content_type).to_string();
            let item_kind = ir::Data::new(&name, Some(ty));
            items.add_item(ir::Item::new(id, size, item_kind));
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&mut self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), traits::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for wasmparser::ExportSectionReader<'a> {
    type ItemsExtra = usize;

    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        idx: usize,
    ) -> Result<(), traits::Error> {
        for (i, exp) in iterate_with_size(self).enumerate() {
            let (exp, size) = exp?;
            let id = Id::entry(idx, i);
            let name = format!("export \"{}\"", exp.field);
            let item_kind = ir::Misc::new(&name);
            items.add_root(ir::Item::new(id, size, item_kind));
        }
        Ok(())
    }

    type EdgesExtra = (&'a SectionIndices, usize);

    fn parse_edges(
        &mut self,
        items: &mut ir::ItemsBuilder,
        (indices, idx): Self::EdgesExtra,
    ) -> Result<(), traits::Error> {
        for (i, exp) in iterate_with_size(self).enumerate() {
            let (exp, _) = exp?;
            let exp_id = Id::entry(idx, i);
            match exp.kind {
                wasmparser::ExternalKind::Function => {
                    items.add_edge(exp_id, indices.functions[exp.index as usize]);
                }
                wasmparser::ExternalKind::Table => {
                    items.add_edge(exp_id, indices.tables[exp.index as usize]);
                }
                wasmparser::ExternalKind::Memory => {
                    items.add_edge(exp_id, indices.memories[exp.index as usize]);
                }
                wasmparser::ExternalKind::Global => {
                    items.add_edge(exp_id, indices.globals[exp.index as usize]);
                }
            }
        }

        Ok(())
    }
}

struct StartSection<'a>(wasmparser::Section<'a>);

impl<'a> Parse<'a> for StartSection<'a> {
    type ItemsExtra = usize;

    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        idx: usize,
    ) -> Result<(), traits::Error> {
        let range = self.0.range();
        let size = (range.end - range.start) as u32;
        let id = Id::section(idx);
        let name = "\"start\" section";
        let item_kind = ir::Misc::new(name);
        items.add_root(ir::Item::new(id, size, item_kind));
        Ok(())
    }

    type EdgesExtra = (&'a SectionIndices, usize);

    fn parse_edges(
        &mut self,
        items: &mut ir::ItemsBuilder,
        (indices, idx): Self::EdgesExtra,
    ) -> Result<(), traits::Error> {
        let f_i = self.0.get_start_section_content()?;
        items.add_edge(Id::section(idx), indices.functions[f_i as usize]);
        Ok(())
    }
}

struct DataCountSection<'a>(wasmparser::Section<'a>);

impl<'a> Parse<'a> for DataCountSection<'a> {
    type ItemsExtra = usize;

    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        idx: usize,
    ) -> Result<(), traits::Error> {
        let range = self.0.range();
        let size = (range.end - range.start) as u32;
        let id = Id::section(idx);
        let name = "\"data count\" section";
        let item_kind = ir::Misc::new(name);
        items.add_root(ir::Item::new(id, size, item_kind));
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&mut self, _items: &mut ir::ItemsBuilder, (): ()) -> Result<(), traits::Error> {
        Ok(())
    }
}

impl<'a> Parse<'a> for wasmparser::ElementSectionReader<'a> {
    type ItemsExtra = usize;

    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        idx: usize,
    ) -> Result<(), traits::Error> {
        for (i, elem) in iterate_with_size(self).enumerate() {
            let (_elem, size) = elem?;
            let id = Id::entry(idx, i);
            let name = format!("elem[{}]", i);
            let item_kind = ir::Misc::new(&name);
            items.add_item(ir::Item::new(id, size, item_kind));
        }
        Ok(())
    }

    type EdgesExtra = (&'a SectionIndices, usize);

    fn parse_edges(
        &mut self,
        items: &mut ir::ItemsBuilder,
        (indices, idx): Self::EdgesExtra,
    ) -> Result<(), traits::Error> {
        for (i, elem) in iterate_with_size(self).enumerate() {
            let (elem, _size) = elem?;
            let elem_id = Id::entry(idx, i);

            match elem.kind {
                wasmparser::ElementKind::Active { table_index, .. } => {
                    items.add_edge(indices.tables[table_index as usize], elem_id);
                }
                wasmparser::ElementKind::Passive(_ty) => {}
            }
            for func_idx in elem.items.get_items_reader()? {
                let func_idx = func_idx?;
                items.add_edge(elem_id, indices.functions[func_idx as usize]);
            }
        }

        Ok(())
    }
}

impl<'a> Parse<'a> for wasmparser::CodeSectionReader<'a> {
    type ItemsExtra = (usize, usize, &'a HashMap<usize, &'a str>);

    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        (idx, imported_functions, names): Self::ItemsExtra,
    ) -> Result<(), traits::Error> {
        for (i, body) in iterate_with_size(self).enumerate() {
            let (_body, size) = body?;
            let id = Id::entry(idx, i);
            let name = names
                .get(&(i + imported_functions))
                .map(ToString::to_string);
            let decorater = format!("code[{}]", i);
            let code = ir::Code::new(name, decorater);
            items.add_item(ir::Item::new(id, size, code));
        }

        Ok(())
    }

    type EdgesExtra = (&'a SectionIndices, usize);

    fn parse_edges(
        &mut self,
        items: &mut ir::ItemsBuilder,
        (indices, idx): Self::EdgesExtra,
    ) -> Result<(), traits::Error> {
        for (b_i, body) in iterate_with_size(self).enumerate() {
            let (body, _size) = body?;
            let body_id = Id::entry(idx, b_i);

            let mut cache = None;
            for op in body.get_operators_reader()? {
                let prev = cache.take();
                match op? {
                    Operator::Call { function_index } => {
                        let f_id = indices.functions[function_index as usize];
                        items.add_edge(body_id, f_id);
                    }

                    // TODO: Rather than looking at indirect calls, need to look
                    // at where the vtables get initialized and/or vtable
                    // indices get pushed onto the stack.
                    Operator::CallIndirect { .. } => continue,

                    Operator::GetGlobal { global_index } | Operator::SetGlobal { global_index } => {
                        let g_id = indices.globals[global_index as usize];
                        items.add_edge(body_id, g_id);
                    }

                    Operator::I32Load { memarg }
                    | Operator::I32Load8S { memarg }
                    | Operator::I32Load8U { memarg }
                    | Operator::I32Load16S { memarg }
                    | Operator::I32Load16U { memarg }
                    | Operator::I64Load { memarg }
                    | Operator::I64Load8S { memarg }
                    | Operator::I64Load8U { memarg }
                    | Operator::I64Load16S { memarg }
                    | Operator::I64Load16U { memarg }
                    | Operator::I64Load32S { memarg }
                    | Operator::I64Load32U { memarg }
                    | Operator::F32Load { memarg }
                    | Operator::F64Load { memarg } => {
                        if let Some(Operator::I32Const { value }) = prev {
                            if let Some(data_id) = items.get_data(value as u32 + memarg.offset) {
                                items.add_edge(body_id, data_id);
                            }
                        }
                    }
                    other => cache = Some(other),
                }
            }
        }

        Ok(())
    }
}

impl<'a> Parse<'a> for wasmparser::DataSectionReader<'a> {
    type ItemsExtra = usize;

    fn parse_items(
        &mut self,
        items: &mut ir::ItemsBuilder,
        idx: usize,
    ) -> Result<(), traits::Error> {
        for (i, d) in iterate_with_size(self).enumerate() {
            let (d, size) = d?;
            let id = Id::entry(idx, i);
            let name = format!("data[{}]", i);
            let item_kind = ir::Data::new(&name, None);
            items.add_item(ir::Item::new(id, size, item_kind));

            // Get the constant address (if any) from the initialization
            // expression.
            if let wasmparser::DataKind::Active { init_expr, .. } = d.kind {
                let mut iter = init_expr.get_operators_reader();
                let offset = match iter.read()? {
                    Operator::I32Const { value } => Some(i64::from(value)),
                    Operator::I64Const { value } => Some(value),
                    _ => None,
                };

                if let Some(off) = offset {
                    let length = d.data.len(); // size of data
                    items.link_data(off, length, id);
                }
            }
        }
        Ok(())
    }

    type EdgesExtra = ();

    fn parse_edges(&mut self, _: &mut ir::ItemsBuilder, _: ()) -> Result<(), traits::Error> {
        Ok(())
    }
}

fn iterate_with_size<'a, S: SectionWithLimitedItems + SectionReader>(
    s: &'a mut S,
) -> impl Iterator<Item = Result<(S::Item, u32), traits::Error>> + 'a {
    let count = s.get_count();
    (0..count).map(move |i| {
        let start = s.original_position();
        let item = s.read()?;
        let size = (s.original_position() - start) as u32;
        if i == count - 1 {
            s.ensure_end()?;
        }
        Ok((item, size))
    })
}

fn ty2str(t: Type) -> &'static str {
    match t {
        Type::I32 => "i32",
        Type::I64 => "i64",
        Type::F32 => "f32",
        Type::F64 => "f64",
        Type::V128 => "v128",
        Type::AnyFunc => "anyfunc",
        Type::AnyRef => "anyref",
        Type::Func | Type::EmptyBlockType => "?",
    }
}
