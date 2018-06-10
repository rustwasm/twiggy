extern crate twiggy_opt;

mod cli;

use std::process;
use cli::structopt::StructOpt;
use cli::failure::Fail;

fn main() {
    let options = twiggy_opt::Options::from_args();
    if let Err(e) = cli::run_twiggy(options) {
        eprintln!("error: {}", e);
        for c in e.causes().skip(1) {
            eprintln!("  caused by: {}", c);
        }
        process::exit(1);
    }
}