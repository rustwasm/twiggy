//! Common traits and types used throughout all of `twiggy`.
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

use anyhow::anyhow;
use std::io;
use std::str::FromStr;
use twiggy_ir as ir;

/// An analysis takes our IR and returns some kind of data results that can be
/// emitted.
pub trait Analyze {
    /// The resulting data from this analysis.
    type Data: Emit;

    /// Run this analysis on the given IR items.
    fn analyze(items: &mut ir::Items) -> anyhow::Result<Self::Data>;
}

/// Selects the parse mode for the input data.
#[derive(Clone, Copy, Debug)]
pub enum ParseMode {
    /// WebAssembly file parse mode.
    Wasm,
    /// DWARF sections parse mode.
    #[cfg(feature = "dwarf")]
    Dwarf,
    /// Automatically determined mode of parsing, e.g. based on file extension.
    Auto,
}

impl Default for ParseMode {
    fn default() -> ParseMode {
        ParseMode::Auto
    }
}

impl FromStr for ParseMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "wasm" => Ok(ParseMode::Wasm),
            #[cfg(feature = "dwarf")]
            "dwarf" => Ok(ParseMode::Dwarf),
            "auto" => Ok(ParseMode::Auto),
            _ => Err(anyhow!("Unknown parse mode: {}", s)),
        }
    }
}

/// The format of the output.
#[derive(Clone, Copy, Debug)]
pub enum OutputFormat {
    /// Human readable text.
    #[cfg(feature = "emit_text")]
    Text,

    // /// Hyper Text Markup Language.
    // Html,
    // /// Graphviz dot format.
    // Dot,
    /// Comma-separated values (CSV) format.
    #[cfg(feature = "emit_csv")]
    Csv,

    /// JavaScript Object Notation format.
    #[cfg(feature = "emit_json")]
    Json,
}

#[cfg(feature = "emit_text")]
#[cfg(feature = "emit_csv")]
#[cfg(feature = "emit_json")]
impl Default for OutputFormat {
    fn default() -> OutputFormat {
        OutputFormat::Text
    }
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[cfg(feature = "emit_text")]
            "text" => Ok(OutputFormat::Text),
            #[cfg(feature = "emit_json")]
            "json" => Ok(OutputFormat::Json),
            #[cfg(feature = "emit_csv")]
            "csv" => Ok(OutputFormat::Csv),
            _ => Err(anyhow!("Unknown output format: {}", s)),
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
        destination: &mut dyn io::Write,
        format: OutputFormat,
    ) -> anyhow::Result<()> {
        match format {
            #[cfg(feature = "emit_text")]
            OutputFormat::Text => self.emit_text(items, destination),
            // OutputFormat::Html => self.emit_html(destination),
            // OutputFormat::Dot => self.emit_dot(destination),
            #[cfg(feature = "emit_csv")]
            OutputFormat::Csv => self.emit_csv(items, destination),
            #[cfg(feature = "emit_json")]
            OutputFormat::Json => self.emit_json(items, destination),
        }
    }

    /// Emit human readable text.
    #[cfg(feature = "emit_text")]
    fn emit_text(&self, items: &ir::Items, destination: &mut dyn io::Write) -> anyhow::Result<()>;

    // /// Emit HTML.
    // fn emit_html(&self, destination: &mut dyn io::Write) -> Result<(), Error>;

    // /// Emit Graphviz's dot format.
    // fn emit_dot(&self, destination: &mut dyn io::Write) -> Result<(), Error>;

    /// Emit CSV.
    #[cfg(feature = "emit_csv")]
    fn emit_csv(&self, items: &ir::Items, destination: &mut dyn io::Write) -> anyhow::Result<()>;

    /// Emit JSON.
    #[cfg(feature = "emit_json")]
    fn emit_json(&self, items: &ir::Items, destination: &mut dyn io::Write) -> anyhow::Result<()>;
}
