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
        for c in Fail::iter_causes(&e) {
            eprintln!("  caused by: {}", c);
        }
        process::exit(1);
    }
}

fn run(opts: &opt::Options) -> Result<(), traits::Error> {
    let mut items = parser::read_and_parse(opts.input(), opts.parse_mode())?;

    let data: Box<dyn traits::Emit> = match opts {
        opt::Options::Top(top) => Box::new(analyze::top(&mut items, top)?),
        opt::Options::Dominators(doms) => Box::new(analyze::dominators(&mut items, doms)?),
        opt::Options::Paths(paths) => Box::new(analyze::paths(&mut items, paths)?),
        opt::Options::Monos(monos) => Box::new(analyze::monos(&mut items, monos)?),
        opt::Options::Garbage(garbo) => Box::new(analyze::garbage(&items, garbo)?),
        opt::Options::Diff(diff) => {
            let mut new_items = parser::read_and_parse(diff.new_input(), opts.parse_mode())?;
            Box::new(analyze::diff(&mut items, &mut new_items, diff)?)
        }
    };

    let mut dest = opts.output_destination().open()?;

    data.emit(&items, &mut *dest, opts.output_format())
}
