# `twiggy dominators`

The `twiggy dominators` sub-command displays the dominator tree of a binary's
call graph.

```
 Retained Bytes │ Retained % │ Dominator Tree
────────────────┼────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
         175726 ┊     14.99% ┊ export "items_parse"
         175712 ┊     14.98% ┊   ⤷ items_parse
         131407 ┊     11.21% ┊       ⤷ twiggy_parser::wasm_parse::<impl twiggy_parser::Parse for wasmparser::readers::module::ModuleReader>::parse_items::h39c45381d868d181
          18492 ┊      1.58% ┊       ⤷ wasmparser::binary_reader::BinaryReader::read_operator::hb1c7cde18e148939
           2677 ┊      0.23% ┊       ⤷ alloc::collections::btree::map::BTreeMap<K,V>::insert::hd2463626e5ac3441
           1349 ┊      0.12% ┊       ⤷ wasmparser::readers::module::ModuleReader::read::hb76af8efd547784f
           1081 ┊      0.09% ┊       ⤷ core::ops::function::impls::<impl core::ops::function::FnOnce<A> for &mut F>::call_once::h1ff7fe5b944492c3
            776 ┊      0.07% ┊       ⤷ <wasmparser::readers::import_section::ImportSectionReader as wasmparser::readers::section_reader::SectionReader>::read::h12903e6d8d4091bd
```
