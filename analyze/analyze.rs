//! Implementations of the analyses that `twiggy` runs on its IR.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate csv;
extern crate petgraph;
extern crate regex;
extern crate twiggy_ir;
extern crate twiggy_opt;
extern crate twiggy_traits;

mod analyses;
mod formats;

pub use analyses::{
    diff::diff, dominators::dominators, garbage::garbage, monos::monos, paths::paths, top::top,
};
