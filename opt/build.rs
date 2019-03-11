use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

fn main() {
    let mut cli = PathBuf::new();
    cli.push(env::var("OUT_DIR").expect("should have OUT_DIR env var"));

    let mut wasm = cli.clone();
    wasm.push("wasm.rs");

    cli.push("cli.rs");

    println!("cargo:rerun-if-changed=./definitions.rs");
    println!("cargo:rerun-if-changed=./build.rs");

    copy_without_lines_matching_pattern("definitions.rs", cli, ".*\\bwasm_bindgen\\b.*");
    copy_without_lines_matching_pattern("definitions.rs", wasm, ".*\\bstructopt\\b.*");
}

fn copy_without_lines_matching_pattern<P1, P2, S>(from: P1, to: P2, pattern: S)
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
    S: AsRef<str>,
{
    let from = from.as_ref();
    let from = fs::File::open(from).expect(&format!("should open `{}` OK", from.display()));
    let from = io::BufReader::new(from);

    let to = to.as_ref();
    let to = fs::File::create(to).expect(&format!("should open `{}` OK", to.display()));
    let mut to = io::BufWriter::new(to);

    let pattern_str = pattern.as_ref();
    let pattern = regex::RegexBuilder::new(pattern_str)
        .case_insensitive(true)
        .build()
        .expect(&format!("should create regex from '{}' OK", pattern_str));

    for line in from.lines() {
        let line = line.expect("should read line OK");

        if pattern.is_match(&line) {
            continue;
        }

        to.write_all(line.as_bytes()).expect("should write line OK");
        to.write_all(b"\n").expect("should write newline OK");
    }
}
