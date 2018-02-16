//! The `svelte` code size profiler.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate failure;

extern crate svelte_analyze as analyze;
extern crate svelte_opt as opt;
extern crate svelte_parser as parser;
extern crate svelte_traits as traits;

use opt::CommonOptions;

/// Run `svelte` with the given options.
pub fn run(opts: opt::Options) -> Result<(), failure::Error> {
    let mut items = parser::parse(opts.input())?;
    let data = match opts {
        opt::Options::Top(ref top) => analyze::top(&mut items, top)?,
        opt::Options::Dominators(ref doms) => analyze::dominators(&mut items, doms)?,
    };
    data.emit(&items, opts.output_destination(), opts.output_format())
}
