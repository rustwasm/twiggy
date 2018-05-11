use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let mut cli = PathBuf::new();
    cli.push(env::var("OUT_DIR").expect("should have OUT_DIR env var"));

    let mut wasm = cli.clone();
    wasm.push("wasm.rs");

    cli.push("cli.rs");

    println!("cargo:rerun-if-changed=./definitions.js");

    run(format!(
        "cat ./definitions.rs | grep -vi wasm_bindgen > '{}'",
        cli.display()
    ));
    run(format!(
        "cat ./definitions.rs | grep -vi structopt > '{}'",
        wasm.display()
    ));
}

fn run<S: AsRef<str>>(cmd: S) {
    let cmd = cmd.as_ref();
    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .status()
        .expect("should run `{}` ok", cmd);
    assert!(status.success());
}
