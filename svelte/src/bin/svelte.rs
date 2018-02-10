extern crate structopt;
extern crate svelte;

use std::process;
use structopt::StructOpt;

fn main() {
    let options = svelte::Options::from_args();
    if let Err(e) = svelte::run(options) {
        eprintln!("error: {}", e);
        process::exit(1);
    }
}
