extern crate structopt;
extern crate svelte;
extern crate svelte_opt as opt;

use std::process;
use structopt::StructOpt;

fn main() {
    let options = opt::Options::from_args();
    if let Err(e) = svelte::run(options) {
        eprintln!("error: {}", e);
        for c in e.causes().skip(1) {
            eprintln!("  caused by: {}", c);
        }
        process::exit(1);
    }
}
