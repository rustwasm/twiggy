//! Options for running `svelte`.

#![deny(missing_debug_implementations)]

#[cfg(feature = "cli")]
#[macro_use]
extern crate structopt;

extern crate svelte_traits as traits;

use std::fs;
use std::io;
use std::path;
use std::str::FromStr;

/// Options for configuring `svelte`.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "cli", derive(StructOpt))]
#[cfg_attr(feature = "cli", structopt(about = "\n`svelte` is a code size profiler.\n\nIt analyzes a binary's call graph to answer questions like:\n\n* Why was this function included in the binary in the first place?\n\n* What is the retained size of this function? I.e. how much space\n  would be saved if I removed it and all the functions that become\n  dead code after its removal.\n\nUse `svelte` to make your binaries slim!"))]
pub enum Options {
    /// List the top code size offenders in a binary.
    #[cfg_attr(feature = "cli", structopt(name = "top"))]
    Top(Top),

    /// Compute and display the dominator tree for a binary's call graph.
    #[cfg_attr(feature = "cli", structopt(name = "dominators"))]
    Dominators(Dominators),

    /// Find and display the call paths to a function in the given binary's call
    /// graph.
    #[cfg_attr(feature = "cli", structopt(name = "paths"))]
    Paths(Paths),
}

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
        }
    }

    fn output_destination(&self) -> &OutputDestination {
        match *self {
            Options::Top(ref top) => top.output_destination(),
            Options::Dominators(ref doms) => doms.output_destination(),
            Options::Paths(ref paths) => paths.output_destination(),
        }
    }

    fn output_format(&self) -> traits::OutputFormat {
        match *self {
            Options::Top(ref top) => top.output_format(),
            Options::Dominators(ref doms) => doms.output_format(),
            Options::Paths(ref paths) => paths.output_format(),
        }
    }
}

/// List the top code size offenders in a binary.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "cli", derive(StructOpt))]
pub struct Top {
    /// The path to the input binary to size profile.
    #[cfg_attr(feature = "cli", structopt(parse(from_os_str)))]
    pub input: path::PathBuf,

    /// The destination to write the output to. Defaults to `stdout`.
    #[cfg_attr(feature = "cli", structopt(short = "o", default_value = "-"))]
    pub output_destination: OutputDestination,

    /// The format the output should be written in.
    #[cfg_attr(feature = "cli", structopt(short = "f", long = "format", default_value = "text"))]
    pub output_format: traits::OutputFormat,

    /// The maximum number of items to display.
    #[cfg_attr(feature = "cli", structopt(short = "n"))]
    pub number: Option<u32>,

    /// Display retaining paths.
    #[cfg_attr(feature = "cli", structopt(short = "r", long = "retaining-paths"))]
    pub retaining_paths: bool,

    /// Sort list by retained
    #[cfg_attr(feature = "cli", structopt(long = "retained"))]
    pub retained: bool,
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

/// Compute and display the dominator tree for a binary's call graph.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "cli", derive(StructOpt))]
pub struct Dominators {
    /// The path to the input binary to size profile.
    #[cfg_attr(feature = "cli", structopt(parse(from_os_str)))]
    pub input: path::PathBuf,

    /// The destination to write the output to. Defaults to `stdout`.
    #[cfg_attr(feature = "cli", structopt(short = "o", default_value = "-"))]
    pub output_destination: OutputDestination,

    /// The format the output should be written in.
    #[cfg_attr(feature = "cli", structopt(short = "f", long = "format", default_value = "text"))]
    pub output_format: traits::OutputFormat,

    /// The maximum depth to print the dominators tree.
    #[cfg_attr(feature = "cli", structopt(short = "d"))]
    pub max_depth: Option<usize>,

    /// The maximum number of rows, regardless of depth in the tree, to display.
    #[cfg_attr(feature = "cli", structopt(short = "r"))]
    pub max_rows: Option<usize>,
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

/// Find and display the call paths to a function in the given binary's call
/// graph.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "cli", derive(StructOpt))]
pub struct Paths {
    /// The path to the input binary to size profile.
    #[cfg_attr(feature = "cli", structopt(parse(from_os_str)))]
    pub input: path::PathBuf,

    /// The functions to find call paths to.
    pub functions: Vec<String>,

    /// The destination to write the output to. Defaults to `stdout`.
    #[cfg_attr(feature = "cli", structopt(short = "o", default_value = "-"))]
    pub output_destination: OutputDestination,

    /// The format the output should be written in.
    #[cfg_attr(feature = "cli", structopt(short = "f", long = "format", default_value = "text"))]
    pub output_format: traits::OutputFormat,

    /// The maximum depth to print the paths.
    #[cfg_attr(feature = "cli", structopt(short = "d", default_value = "10"))]
    pub max_depth: usize,

    /// The maximum number of paths, regardless of depth in the tree, to display.
    #[cfg_attr(feature = "cli", structopt(short = "r", default_value = "10"))]
    pub max_paths: usize,
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
                Box::new(io::BufWriter::new(fs::File::open(path)?)) as Box<io::Write>
            }
            OutputDestination::Stdout => Box::new(io::stdout()) as Box<io::Write>,
        })
    }
}
