//! The `svelte` code size profiler.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate failure;
extern crate structopt;
extern crate svelte_analyze as analyze;
extern crate svelte_opt as opt;
extern crate svelte_parser as parser;
extern crate svelte_traits as traits;

use failure::Fail;
use opt::CommonOptions;
use std::process;
use structopt::StructOpt;

fn main() {
    let options = opt::Options::from_args();
    if let Err(e) = run(options) {
        eprintln!("error: {}", e);
        for c in e.causes().skip(1) {
            eprintln!("  caused by: {}", c);
        }
        process::exit(1);
    }
}

fn run(opts: opt::Options) -> Result<(), traits::Error> {
    let mut items = parser::read_and_parse(opts.input())?;

    let data = match opts {
        opt::Options::Top(ref top) => analyze::top(&mut items, top)?,
        opt::Options::Dominators(ref doms) => analyze::dominators(&mut items, doms)?,
        opt::Options::Paths(ref paths) => analyze::paths(&mut items, paths)?,
    };

    let mut dest = opts.output_destination().open()?;

    data.emit(&items, &mut *dest, opts.output_format())
}
