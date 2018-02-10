//! The `svelte` code size profiler.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate failure;
extern crate svelte_analyze as analyze;
extern crate svelte_ir as ir;
extern crate svelte_parser as parser;
extern crate svelte_traits as traits;

use std::path;

/// Options for controlling `svelte`.
#[derive(Clone, Debug)]
pub struct Options {
    /// The input file that we wish to size profile.
    pub input: path::PathBuf,

    /// The destination to write the output to.
    ///
    /// If none is provided, then `stdout`.
    pub output: Option<path::PathBuf>,

    /// The format the output should be written in.
    pub output_format: traits::OutputFormat,

    /// The analysis to run.
    pub analysis: Analysis,
}

/// The analysis to run.
#[derive(Clone, Debug)]
pub enum Analysis {
    WhoCalls,
    TopRetained,
    TopSelf,
}



/// Run `svelte` with the given options.
pub fn run(mut opts: Options) -> Result<(), failure::Error> {
    let ir = parser::parse(&opts.input)?;
    let data = analyze::analyze(ir, &opts.analysis)?;

    let mut dest = match opts.output {
        Some(path) => Box::new(fs::Open(path)?) as Box<io::Write>,
        None => Box::new(io::stdout()) as Box<io::Write>,
    };

    data.emit(&mut dest, opts.output_format)
}
