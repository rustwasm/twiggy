//! Implementations of the analyses that `svelte` runs on its IR.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate failure;
extern crate svelte_ir as ir;
extern crate svelte_opt as opt;
extern crate svelte_traits as traits;

use failure::ResultExt;
use std::cmp;
use std::fmt;

#[derive(Debug, Clone, Copy)]
enum Align {
    Left,
    Right,
}

#[derive(Debug, Clone)]
struct Table {
    header: Vec<(Align, String)>,
    rows: Vec<Vec<String>>,
}

impl Table {
    fn with_header(header: Vec<(Align, String)>) -> Table {
        assert!(!header.is_empty());
        Table {
            header,
            rows: vec![],
        }
    }

    fn add_row(&mut self, row: Vec<String>) {
        assert_eq!(self.header.len(), row.len());
        self.rows.push(row);
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut maxs: Vec<_> = self.header.iter().map(|h| h.1.len()).collect();

        for row in &self.rows {
            for (i, x) in row.iter().enumerate() {
                maxs[i] = cmp::max(maxs[i], x.len());
            }
        }

        let last = self.header.len() - 1;

        for (i, h) in self.header.iter().map(|h| &h.1).enumerate() {
            if i == 0 {
                write!(f, " ")?;
            } else {
                write!(f, " │ ")?;
            }

            write!(f, "{}", h)?;
            if i != last {
                for _ in 0..maxs[i] - h.len() {
                    write!(f, " ")?;
                }
            }
        }
        write!(f, "\n")?;

        for i in 0..self.header.len() {
            if i == 0 {
                write!(f, "─")?;
            } else {
                write!(f, "─┼─")?;
            }
            for _ in 0..maxs[i] {
                write!(f, "─")?;
            }
        }
        write!(f, "\n")?;

        for row in &self.rows {
            for (i, (x, align)) in row.iter().zip(self.header.iter().map(|h| h.0)).enumerate() {
                if i == 0 {
                    write!(f, " ")?;
                } else {
                    write!(f, " ┊ ")?;
                }

                match align {
                    Align::Left => {
                        write!(f, "{}", x)?;
                        if i != last {
                            for _ in 0..maxs[i] - x.len() {
                                write!(f, " ")?;
                            }
                        }
                    }
                    Align::Right => {
                        for _ in 0..maxs[i] - x.len() {
                            write!(f, " ")?;
                        }
                        write!(f, "{}", x)?;
                    }
                }
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}

struct Top {
    items: Vec<ir::Id>,
}

impl traits::Emit for Top {
    fn emit_text(
        &self,
        items: &ir::Items,
        dest: &opt::OutputDestination,
    ) -> Result<(), failure::Error> {
        let mut dest = dest.open().context("could not open output destination")?;

        let mut table = Table::with_header(vec![
            (Align::Right, "Shallow Bytes".to_string()),
            (Align::Right, "Shallow %".to_string()),
            (Align::Left, "Item".to_string()),
        ]);

        for &id in &self.items {
            let item = &items[id];
            let size = item.size();
            let size_percent = (size as f64) / (items.size() as f64) * 100.0;
            table.add_row(vec![
                size.to_string(),
                format!("{:.2}%", size_percent),
                item.name().to_string(),
            ]);
        }

        write!(&mut dest, "{}", &table)?;
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
    Ok(Box::new(Top { items }) as Box<traits::Emit>)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
