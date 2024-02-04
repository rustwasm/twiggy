//! The `twiggy` code size profiler.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

use std::process;
use structopt::StructOpt;
use twiggy_analyze as analyze;
use twiggy_opt::{self as opt, CommonCliOptions};
use twiggy_parser as parser;

fn main() {
    let options = opt::Options::from_args();
    if let Err(e) = run(&options) {
        eprintln!("error: {}", e);
        process::exit(1);
    }
}

fn run(opts: &opt::Options) -> anyhow::Result<()> {
    let mut items = parser::read_and_parse(opts.input(), opts.parse_mode())?;

    let data = match opts {
        opt::Options::Top(ref top) => analyze::top(&mut items, top)?,
        opt::Options::Dominators(ref doms) => analyze::dominators(&mut items, doms)?,
        opt::Options::Paths(ref paths) => analyze::paths(&mut items, paths)?,
        opt::Options::Monos(ref monos) => analyze::monos(&mut items, monos)?,
        opt::Options::Garbage(ref garbo) => analyze::garbage(&items, garbo)?,
        opt::Options::Diff(ref diff) => {
            let mut new_items = parser::read_and_parse(diff.new_input(), opts.parse_mode())?;
            analyze::diff(&mut items, &mut new_items, diff)?
        }
    };

    // eprintln!("DominatorTree");
    // for (key, values) in items.dominator_tree().iter() {
    //     let item = &items[*key];
    //     let item_name = item.name();
    //     let retained_size = items.retained_size(*key);
    //     eprintln!(
    //         "{} (size={}, retained={}):",
    //         item_name,
    //         item.size(),
    //         retained_size
    //     );
    //     for child in values.iter() {
    //         let item = &items[*child];
    //         let retained_size = items.retained_size(*child);
    //         let child_name = item.name();
    //         eprintln!(
    //             "  {} (size={}, retained={})",
    //             child_name,
    //             item.size(),
    //             retained_size
    //         );
    //     }
    // }

    // eprintln!("\n\nEdges");
    // for item in items.iter() {
    //     let item_name = item.name();
    //     let retained_size = items.retained_size(item.id());
    //     eprintln!(
    //         "{} (size={}, retained={}):",
    //         item_name,
    //         item.size(),
    //         retained_size
    //     );

    //     for child in items.neighbors(item.id()) {
    //         let item = &items[child];
    //         let retained_size = items.retained_size(child);
    //         let child_name = item.name();
    //         eprintln!(
    //             "  {} (size={}, retained={})",
    //             child_name,
    //             item.size(),
    //             retained_size
    //         );
    //     }
    // }

    let mut dest = opts.output_destination().open()?;

    data.emit(&items, &mut *dest, opts.output_format())
}
