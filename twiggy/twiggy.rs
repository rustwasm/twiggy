//! The `twiggy` code size profiler.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

use std::process;

use failure::Fail;
use structopt::StructOpt;

use twiggy_analyze as analyze;
use twiggy_opt::{self as opt, CommonCliOptions};
use twiggy_parser as parser;
use twiggy_traits as traits;

fn main() {
    let options = opt::Options::from_args();
    if let Err(e) = run(&options) {
        eprintln!("error: {}", e);
        for c in <dyn Fail>::iter_causes(&e) {
            eprintln!("  caused by: {}", c);
        }
        process::exit(1);
    }
}

fn run(opts: &opt::Options) -> Result<(), traits::Error> {
    let mut items = parser::read_and_parse(opts.input(), opts.parse_mode())?;

    let data = match opts {
        opt::Options::Top(ref top) => analyze::top(&mut items, top)?,
        opt::Options::Dominators(ref doms) => analyze::dominators(&mut items, doms)?,
        opt::Options::Paths(ref paths) => analyze::paths(&mut items, paths)?,
        opt::Options::Monos(ref monos) => analyze::monos(&mut items, monos)?,
        opt::Options::Garbage(ref garbo) => analyze::garbage(&items, garbo)?,
        opt::Options::Diff(ref diff) => {
            let mut new_items = parser::read_and_parse(diff.new_input(), opts.parse_mode())?;
            analyze::diff(&mut items, &mut new_items, diff)?
        }
    };

    let mut dest = opts.output_destination().open()?;

    data.emit(&items, &mut *dest, opts.output_format())
}
