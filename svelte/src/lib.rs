//! The `svelte` code size profiler.

#![deny(missing_debug_implementations)]

#[macro_use]
extern crate structopt_derive;

#[macro_use]
extern crate failure;

extern crate structopt;
extern crate svelte_analyze as analyze;
extern crate svelte_ir as ir;
extern crate svelte_parser as parser;
extern crate svelte_traits as traits;

use std::path;
use std::str::FromStr;

/// Options for controlling `svelte`.
#[derive(Clone, Debug, StructOpt)]
pub enum Options {
    /// List the top code size offenders in a binary.
    #[structopt(name = "top")]
    Top {
        /// The path to the input binary to size profile.
        #[structopt(parse(from_os_str))]
        input: path::PathBuf,

        /// The destination to write the output to. Defaults to `stdout`.
        #[structopt(short = "o", default_value = "-")]
        output_destination: traits::OutputDestination,

        /// The format the output should be written in.
        #[structopt(short = "f", long = "format", default_value = "text")]
        output_format: traits::OutputFormat,

        /// The maximum number of items to display.
        #[structopt(short = "n")]
        number: Option<u32>,

        /// Display retaining paths.
        #[structopt(short = "r", long = "retaining-paths")]
        retaining_paths: bool,

        /// Choose how to sort the list. Choices are "shallow" or "retained".
        #[structopt(short = "s", long = "sort-by", default_value = "shallow")]
        sort_by: SortBy,
    },
}

impl Options {
    fn input(&self) -> &path::Path {
        match *self {
            Options::Top { ref input, .. } => input,
        }
    }

    fn output_destination(&self) -> &traits::OutputDestination {
        match *self {
            Options::Top {
                ref output_destination,
                ..
            } => output_destination,
        }
    }

    fn output_format(&self) -> traits::OutputFormat {
        match *self {
            Options::Top { output_format, .. } => output_format,
        }
    }
}

/// Whether to sort by shallow or retained size.
#[derive(Clone, Debug, StructOpt)]
pub enum SortBy {
    Shallow,
    Retained,
}

impl FromStr for SortBy {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<SortBy, failure::Error> {
        match s {
            "shallow" => Ok(SortBy::Shallow),
            "retained" => Ok(SortBy::Retained),
            _ => bail!("unknown sort order: '{}'", s),
        }
    }
}

/// Run `svelte` with the given options.
pub fn run(opts: Options) -> Result<(), failure::Error> {
    let mut items = parser::parse(opts.input())?;
    let data = match opts {
        Options::Top { .. } => analyze::top(&mut items)?,
    };
    data.emit(opts.output_destination(), opts.output_format())
}
