//! Implementations of the analyses that `twiggy` runs on its IR.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

mod analyses;
mod formats;

pub use analyses::{
    diff::diff,
    dominators::dominators,
    garbage::garbage,
    monos::monos,
    paths::paths,
    top::{top, Top},
};
