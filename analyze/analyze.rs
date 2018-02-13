//! Implementations of the analyses that `svelte` runs on its IR.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

#[macro_use]
extern crate failure;
extern crate svelte_ir as ir;
extern crate svelte_opt as opt;
extern crate svelte_traits as traits;

use failure::ResultExt;
use std::cmp;

struct Top(Vec<ir::Id>);

impl traits::Emit for Top {
    fn emit_text(
        &self,
        items: &ir::Items,
        dest: &opt::OutputDestination,
    ) -> Result<(), failure::Error> {
        let mut dest = dest.open().context("could not open output destination")?;

        let mut max_size = "Size".len();
        let table: Vec<_> = self.0
            .iter()
            .cloned()
            .map(|id| {
                let item = &items[id];

                let size = item.size().to_string();
                max_size = cmp::max(size.len(), max_size);

                (size, item.name())
            })
            .collect();

        write!(&mut dest, "Size")?;
        for _ in 0..max_size - "Size".len() {
            write!(&mut dest, " ")?;
        }

        writeln!(&mut dest, " | Item")?;

        for _ in 0..max_size {
            write!(&mut dest, "-")?;
        }
        writeln!(
            &mut dest,
            "-+-----------------------------------------------------"
        )?;

        for (size, name) in table {
            for _ in 0..max_size - size.len() {
                write!(&mut dest, " ")?;
            }
            writeln!(&mut dest, "{} | {}", size, name)?;
        }

        Ok(())
    }
}

/// Run the `top` analysis on the given IR items.
pub fn top(items: &mut ir::Items, opts: &opt::Top) -> Result<Box<traits::Emit>, failure::Error> {
    let mut items: Vec<_> = items.iter().collect();
    items.sort_unstable_by(|a, b| b.size().cmp(&a.size()));
    if let Some(n) = opts.number {
        items.truncate(n as usize);
    }
    let items: Vec<_> = items.into_iter().map(|i| i.id()).collect();
    Ok(Box::new(Top(items)) as Box<traits::Emit>)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
