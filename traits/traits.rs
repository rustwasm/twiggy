//! Common traits and types used throughout all of `svelte`.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate failure;
extern crate svelte_ir as ir;
extern crate svelte_opt as opt;

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
        destination: &opt::OutputDestination,
        format: opt::OutputFormat,
    ) -> Result<(), failure::Error> {
        match format {
            opt::OutputFormat::Text => self.emit_text(destination),
            // OutputFormat::Html => self.emit_html(destination),
            // OutputFormat::Dot => self.emit_dot(destination),
            // OutputFormat::Csv => self.emit_csv(destination),
            // OutputFormat::Json => self.emit_json(destination),
        }
    }

    /// Emit human readable text.
    fn emit_text(&self, destination: &opt::OutputDestination) -> Result<(), failure::Error>;

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
