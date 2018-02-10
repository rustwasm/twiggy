//! Common traits and types used throughout all of `svelte`.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

#[macro_use]
extern crate failure;
extern crate svelte_ir as ir;

use std::fs;
use std::io;
use std::path;
use std::str::FromStr;

/// The analysis to run.
#[derive(Clone, Debug)]
pub enum Analysis {
    /// List the top functions using the most code size.
    Top,
    // WhoCalls,
    // TopRetained,
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
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, failure::Error> {
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
    pub fn open(&self) -> Result<Box<io::Write>, failure::Error> {
        Ok(match *self {
            OutputDestination::Path(ref path) => Box::new(fs::File::open(path)?) as Box<io::Write>,
            OutputDestination::Stdout => Box::new(io::stdout()) as Box<io::Write>,
        })
    }
}

/// The format of the output.
#[derive(Clone, Copy, Debug)]
pub enum OutputFormat {
    /// Human readable text.
    Text,
    // /// Hyper Text Markup Language.
    // Html,

    // /// Graphviz dot format.
    // Dot,

    // /// Comma-separated values (CSV) format.
    // Csv,

    // /// JavaScript Object Notation format.
    // Json,
}

impl Default for OutputFormat {
    fn default() -> OutputFormat {
        OutputFormat::Text
    }
}

impl FromStr for OutputFormat {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, failure::Error> {
        match s {
            "text" => Ok(OutputFormat::Text),
            _ => bail!("Unknown output format: {}", s),
        }
    }
}

/// An analysis takes our IR and returns some kind of data results that can be
/// emitted.
pub trait Analyze {
    /// The resulting data from this analysis.
    type Data: Emit;

    /// Run this analysis on the given IR items.
    fn analyze(items: &mut ir::Items) -> Result<Self::Data, failure::Error>;
}

/// Anything that can write itself in the given output format to the given
/// destination.
pub trait Emit {
    /// Emit this thing to the given destination in the given output format.
    fn emit(
        &self,
        destination: &OutputDestination,
        format: OutputFormat,
    ) -> Result<(), failure::Error> {
        match format {
            OutputFormat::Text => self.emit_text(destination),
            // OutputFormat::Html => self.emit_html(destination),
            // OutputFormat::Dot => self.emit_dot(destination),
            // OutputFormat::Csv => self.emit_csv(destination),
            // OutputFormat::Json => self.emit_json(destination),
        }
    }

    /// Emit human readable text.
    fn emit_text(&self, destination: &OutputDestination) -> Result<(), failure::Error>;

    // /// Emit HTML.
    // fn emit_html(&self, destination: &mut io::Write) -> Result<(), failure::Error>;

    // /// Emit Graphviz's dot format.
    // fn emit_dot(&self, destination: &mut io::Write) -> Result<(), failure::Error>;

    // /// Emit CSV.
    // fn emit_csv(&self, destination: &mut io::Write) -> Result<(), failure::Error>;

    // /// Emit JSON.
    // fn emit_json(&self, destination: &mut io::Write) -> Result<(), failure::Error>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
