//! The `twiggy` code size profiler.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

pub(crate) extern crate failure;
pub(crate) extern crate structopt;
pub(crate) extern crate twiggy_analyze as analyze;
pub(crate) extern crate twiggy_opt as opt;
pub(crate) extern crate twiggy_parser as parser;
pub(crate) extern crate twiggy_traits as traits;

use self::opt::CommonCliOptions;

pub(crate) fn run_twiggy(opts: opt::Options) -> Result<(), traits::Error> {
    let mut items = parser::read_and_parse(opts.input())?;

    let data = match opts {
        opt::Options::Top(ref top) => analyze::top(&mut items, top)?,
        opt::Options::Dominators(ref doms) => analyze::dominators(&mut items, doms)?,
        opt::Options::Paths(ref paths) => analyze::paths(&mut items, paths)?,
        opt::Options::Monos(ref monos) => analyze::monos(&mut items, monos)?,
        opt::Options::Garbage(ref garbo) => analyze::garbage(&mut items, garbo)?,
        opt::Options::Diff(ref diff) => {
            let mut new_items = parser::read_and_parse(diff.new_input())?;
            analyze::diff(&mut items, &mut new_items, diff)?
        }
    };

    let mut dest = opts.output_destination().open()?;

    data.emit(&items, &mut *dest, opts.output_format())
}
