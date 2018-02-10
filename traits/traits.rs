//! Common traits used throughout `svelte`.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

#[macro_use]
extern crate failure;
extern crate svelte_ir as ir;

use std::io;

/// The format of the output.
#[derive(Clone, Copy, Debug)]
pub enum OutputFormat {
    /// Human readable text.
    Text,

    /// Hyper Text Markup Language.
    Html,

    /// Graphviz dot format.
    Dot,

    /// Comma-separated values (CSV) format.
    Csv,

    /// JavaScript Object Notation format.
    Json,
}

impl Default for OutputFormat {
    fn default() -> OutputFormat {
        OutputFormat::Text
    }
}

/// An analysis takes our IR and returns some kind of data results that can be
/// emitted.
pub trait Analyze<T, U> {
    /// Run this analysis on the given IR items.
    fn analyze(items: ir::Items<T>) -> Result<ir::Items<U>, failure::Error>;
}

/// Anything that can write itself in the given output format to the given
/// destination.
pub trait Emit {
    /// Emit this thing to the given destination in the given output format.
    fn emit(
        &self,
        destination: &mut io::Write,
        format: OutputFormat,
    ) -> Result<(), failure::Error> {
        match format {
            OutputFormat::Text => self.emit_text(destination),
            OutputFormat::Html => self.emit_html(destination),
            OutputFormat::Dot => self.emit_dot(destination),
            OutputFormat::Csv => self.emit_csv(destination),
            OutputFormat::Json => self.emit_json(destination),
        }
    }

    /// Emit human readable text.
    fn emit_text(&self, destination: &mut io::Write) -> Result<(), failure::Error>;

    /// Emit HTML.
    fn emit_html(&self, destination: &mut io::Write) -> Result<(), failure::Error>;

    /// Emit Graphviz's dot format.
    fn emit_dot(&self, destination: &mut io::Write) -> Result<(), failure::Error>;

    /// Emit CSV.
    fn emit_csv(&self, destination: &mut io::Write) -> Result<(), failure::Error>;

    /// Emit JSON.
    fn emit_json(&self, destination: &mut io::Write) -> Result<(), failure::Error>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
