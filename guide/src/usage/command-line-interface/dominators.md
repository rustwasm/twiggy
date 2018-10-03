# `twiggy dominators`

The `twiggy dominators` sub-command displays the dominator tree of a binary's
call graph.

```
$ twiggy dominators path/to/input.wasm
 Retained Bytes │ Retained % │ Dominator Tree
────────────────┼────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
         284691 ┊     47.92% ┊ export "items_parse"
         284677 ┊     47.91% ┊   ⤷ func[17]
         284676 ┊     47.91% ┊       ⤷ items_parse
         128344 ┊     21.60% ┊           ⤷ func[47]
         128343 ┊     21.60% ┊               ⤷ twiggy_parser::wasm::<impl twiggy_parser::Parse<'a> for parity_wasm::elements::module::Module>::parse_items::h033e4aa1338b4363
          98403 ┊     16.56% ┊           ⤷ func[232]
          98402 ┊     16.56% ┊               ⤷ twiggy_ir::demangle::h7fb5cfffc912bc2f
          34206 ┊      5.76% ┊           ⤷ func[20]
          34205 ┊      5.76% ┊               ⤷ <parity_wasm::elements::section::Section as parity_wasm::elements::Deserialize>::deserialize::hdd814798147ca8dc
           2855 ┊      0.48% ┊           ⤷ func[552]
           2854 ┊      0.48% ┊               ⤷ <alloc::btree::map::BTreeMap<K, V>>::insert::he64f84697ccf122d
           1868 ┊      0.31% ┊           ⤷ func[53]
           1867 ┊      0.31% ┊               ⤷ twiggy_ir::ItemsBuilder::finish::h1b98f5cc4c80137d
```
