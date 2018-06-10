#[macro_use]
extern crate structopt;

mod cli;

use cli::failure::Fail;
use cli::opt;
use std::process;
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
#[structopt(bin_name = "cargo")]
pub enum App {
    #[structopt(name = "twiggy")]
    Opt(opt::Options),
}

fn main() {
    let App::Opt(opts) = App::from_args();
    if let Err(e) = cli::run_twiggy(opts) {
        eprintln!("error: {}", e);
        for c in e.causes().skip(1) {
            eprintln!("  caused by: {}", c);
        }
        process::exit(1);
    }
}
