//! Common traits and types used throughout all of `twiggy`.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

#[macro_use]
extern crate failure;

extern crate parity_wasm as wasm;
extern crate twiggy_ir as ir;

use std::fmt;
use std::io;
use std::str::FromStr;

/// An error that ocurred in `twiggy` when parsing, analyzing, or emitting
/// items.
#[derive(Debug)]
pub struct Error {
    inner: Box<ErrorInner>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl failure::Fail for Error {
    fn cause(&self) -> Option<&failure::Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&failure::Backtrace> {
        self.inner.backtrace()
    }
}

#[derive(Debug, Fail)]
enum ErrorInner {
    #[fail(display = "{}", _0)]
    Msg(String),

    #[fail(display = "I/O error: {}", _0)]
    Io(#[cause] io::Error),

    #[fail(display = "WASM error: {}", _0)]
    Wasm(#[cause] wasm::elements::Error),

    #[fail(display = "formatting error: {}", _0)]
    Fmt(#[cause] fmt::Error),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error {
            inner: Box::new(ErrorInner::Io(e)),
        }
    }
}

impl From<wasm::elements::Error> for Error {
    fn from(e: wasm::elements::Error) -> Error {
        Error {
            inner: Box::new(ErrorInner::Wasm(e)),
        }
    }
}

impl From<fmt::Error> for Error {
    fn from(e: fmt::Error) -> Error {
        Error {
            inner: Box::new(ErrorInner::Fmt(e)),
        }
    }
}

impl Error {
    /// Create an error with the given message.
    pub fn with_msg<S: Into<String>>(msg: S) -> Error {
        Error {
            inner: Box::new(ErrorInner::Msg(msg.into())),
        }
    }
}

#[test]
fn size_of_error_is_one_word() {
    use std::mem;
    assert_eq!(mem::size_of::<Error>(), mem::size_of::<usize>());
}

/// An analysis takes our IR and returns some kind of data results that can be
/// emitted.
pub trait Analyze {
    /// The resulting data from this analysis.
    type Data: Emit;

    /// Run this analysis on the given IR items.
    fn analyze(items: &mut ir::Items) -> Result<Self::Data, Error>;
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
    /// JavaScript Object Notation format.
    Json,
}

impl Default for OutputFormat {
    fn default() -> OutputFormat {
        OutputFormat::Text
    }
}

impl FromStr for OutputFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            _ => Err(Error::with_msg(format!("Unknown output format: {}", s))),
        }
    }
}

/// Anything that can write itself in the given output format to the given
/// destination.
pub trait Emit {
    /// Emit this thing to the given destination in the given output format.
    fn emit(
        &self,
        items: &ir::Items,
        destination: &mut io::Write,
        format: OutputFormat,
    ) -> Result<(), Error> {
        match format {
            OutputFormat::Text => self.emit_text(items, destination),
            // OutputFormat::Html => self.emit_html(destination),
            // OutputFormat::Dot => self.emit_dot(destination),
            // OutputFormat::Csv => self.emit_csv(destination),
            OutputFormat::Json => self.emit_json(items, destination),
        }
    }

    /// Emit human readable text.
    fn emit_text(&self, items: &ir::Items, destination: &mut io::Write) -> Result<(), Error>;

    // /// Emit HTML.
    // fn emit_html(&self, destination: &mut io::Write) -> Result<(), Error>;

    // /// Emit Graphviz's dot format.
    // fn emit_dot(&self, destination: &mut io::Write) -> Result<(), Error>;

    // /// Emit CSV.
    // fn emit_csv(&self, destination: &mut io::Write) -> Result<(), Error>;

    /// Emit JSON.
    fn emit_json(&self, items: &ir::Items, destination: &mut io::Write) -> Result<(), Error>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
