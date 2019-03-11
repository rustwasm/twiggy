#![cfg(target_arch = "wasm32")]
#![cfg(feature = "emit_json")]

use twiggy_analyze as analyze;
use twiggy_ir as ir;
use twiggy_opt as opt;
use twiggy_parser as parser;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Items {
    items: ir::Items,
}

#[wasm_bindgen]
impl Items {
    pub fn parse(data: &[u8]) -> Items {
        let items = parser::parse(data).unwrap();
        Items { items }
    }

    pub fn top(&mut self, options: &opt::Top) -> String {
        let top = analyze::top(&mut self.items, options).unwrap();
        let mut buf = Vec::new();
        top.emit_json(&self.items, &mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    pub fn dominators(&mut self, options: &opt::Dominators) -> String {
        let dominators = analyze::dominators(&mut self.items, options).unwrap();
        let mut buf = Vec::new();
        dominators.emit_json(&self.items, &mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    pub fn paths(&mut self, options: &opt::Paths) -> String {
        let paths = analyze::paths(&mut self.items, options).unwrap();
        let mut buf = Vec::new();
        paths.emit_json(&self.items, &mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    pub fn monos(&mut self, options: &opt::Monos) -> String {
        let monos = analyze::monos(&mut self.items, options).unwrap();
        let mut buf = Vec::new();
        monos.emit_json(&self.items, &mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    pub fn diff(&mut self, new_items: &mut Items, options: &opt::Diff) -> String {
        let diff = analyze::diff(&mut self.items, &mut new_items.items, options).unwrap();
        let mut buf = Vec::new();
        diff.emit_json(&self.items, &mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }
}
