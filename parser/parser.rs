//! Parses binaries into `svelte_ir::Items`.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

#[macro_use]
extern crate failure;
extern crate svelte_ir as ir;

use std::path;

/// Parse the file at the given path into IR items.
pub fn parse<P: AsRef<path::Path>>(path: P) -> Result<ir::Items, failure::Error> {
    bail!("not yet implemented")
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
