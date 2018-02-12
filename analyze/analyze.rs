//! Implementations of the analyses that `svelte` runs on its IR.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

#[macro_use]
extern crate failure;
extern crate svelte_ir as ir;
extern crate svelte_opt as opt;
extern crate svelte_traits as traits;

/// Run the `top` analysis on the given IR items.
pub fn top(_items: &mut ir::Items, _opts: &opt::Top) -> Result<Box<traits::Emit>, failure::Error> {
    bail!("not yet implemented")
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
