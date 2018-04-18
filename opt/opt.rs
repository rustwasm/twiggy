//! Options for running `twiggy`.

#![deny(missing_debug_implementations)]
#![cfg_attr(feature = "wasm", feature(proc_macro, wasm_custom_section, wasm_import_module))]

#[macro_use]
extern crate cfg_if;

extern crate twiggy_traits as traits;

cfg_if! {
    if #[cfg(feature = "cli")] {
        #[macro_use]
        extern crate structopt;
        include!(concat!(env!("OUT_DIR"), "/cli.rs"));
    } else if #[cfg(feature = "wasm")] {
        extern crate wasm_bindgen;
        use wasm_bindgen::prelude::*;
        include!(concat!(env!("OUT_DIR"), "/wasm.rs"));
    } else {
        compile_error!("Must enable one of either `cli` or `wasm` features");
    }
}

use std::fs;
use std::io;
use std::path;
use std::str::FromStr;
use std::u32;

/// Options that are common to all commands.
pub trait CommonOptions {
    /// Get the input file path.
    fn input(&self) -> &path::Path;

    /// Get the output destination.
    fn output_destination(&self) -> &OutputDestination;

    /// Get the output format.
    fn output_format(&self) -> traits::OutputFormat;
}

impl CommonOptions for Options {
    fn input(&self) -> &path::Path {
        match *self {
            Options::Top(ref top) => top.input(),
            Options::Dominators(ref doms) => doms.input(),
            Options::Paths(ref paths) => paths.input(),
            Options::Monos(ref monos) => monos.input(),
        }
    }

    fn output_destination(&self) -> &OutputDestination {
        match *self {
            Options::Top(ref top) => top.output_destination(),
            Options::Dominators(ref doms) => doms.output_destination(),
            Options::Paths(ref paths) => paths.output_destination(),
            Options::Monos(ref monos) => monos.output_destination(),
        }
    }

    fn output_format(&self) -> traits::OutputFormat {
        match *self {
            Options::Top(ref top) => top.output_format(),
            Options::Dominators(ref doms) => doms.output_format(),
            Options::Paths(ref paths) => paths.output_format(),
            Options::Monos(ref monos) => monos.output_format(),
        }
    }
}

impl CommonOptions for Top {
    fn input(&self) -> &path::Path {
        &self.input
    }

    fn output_destination(&self) -> &OutputDestination {
        &self.output_destination
    }

    fn output_format(&self) -> traits::OutputFormat {
        self.output_format
    }
}

impl CommonOptions for Dominators {
    fn input(&self) -> &path::Path {
        &self.input
    }

    fn output_destination(&self) -> &OutputDestination {
        &self.output_destination
    }

    fn output_format(&self) -> traits::OutputFormat {
        self.output_format
    }
}

impl CommonOptions for Paths {
    fn input(&self) -> &path::Path {
        &self.input
    }

    fn output_destination(&self) -> &OutputDestination {
        &self.output_destination
    }

    fn output_format(&self) -> traits::OutputFormat {
        self.output_format
    }
}

impl CommonOptions for Monos {
    fn input(&self) -> &path::Path {
        &self.input
    }

    fn output_destination(&self) -> &OutputDestination {
        &self.output_destination
    }

    fn output_format(&self) -> traits::OutputFormat {
        self.output_format
    }
}

/// Where to output results.
#[derive(Clone, Debug)]
pub enum OutputDestination {
    /// Emit the results to `stdout`.
    Stdout,

    /// Write the results to a file at the given path.
    Path(path::PathBuf),
}

impl Default for OutputDestination {
    fn default() -> OutputDestination {
        OutputDestination::Stdout
    }
}

impl FromStr for OutputDestination {
    type Err = traits::Error;

    fn from_str(s: &str) -> Result<Self, traits::Error> {
        if s == "-" {
            Ok(OutputDestination::Stdout)
        } else {
            let path = path::PathBuf::from(s.to_string());
            Ok(OutputDestination::Path(path))
        }
    }
}

impl OutputDestination {
    /// Open the output destination as an `io::Write`.
    pub fn open(&self) -> Result<Box<io::Write>, traits::Error> {
        Ok(match *self {
            OutputDestination::Path(ref path) => {
                Box::new(io::BufWriter::new(fs::File::create(path)?)) as Box<io::Write>
            }
            OutputDestination::Stdout => Box::new(io::stdout()) as Box<io::Write>,
        })
    }
}
