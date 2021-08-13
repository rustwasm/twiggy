use anyhow::{anyhow, Result};
use rustop::opts;
use std::{io::Write, process::Command};

fn main() -> Result<()> {
    let (args, _) = opts! {
        synopsis "Twiggy CI job runner";
        opt wasm:bool, desc:"Run wasm jobs";
        opt test:bool, desc:"Run tests";
    }
    .parse_or_exit();

    if args.wasm && args.test {
        anyhow!("Choose only one mode!");
    }

    if args.wasm {
        Command::new("rustup")
            .args(["update", "nightly"])
            .output()?;

        Command::new("rustup")
            .args([
                "target",
                "add",
                "wasm32-unknown-unknown",
                "--toolchain",
                "nightly",
            ])
            .output()
            .map(|output| -> Result<()> {
                std::io::stdout().write_all(&output.stdout).unwrap();
                std::io::stderr().write_all(&output.stderr).unwrap();

                if !output.status.success() {
                    Err(anyhow!("Failed to add wasm32-unknown-unknown!"))
                } else {
                    Ok(())
                }
            })??;

        Command::new("cargo")
            .current_dir("./wasm-api")
            .args([
                "+nightly",
                "build",
                "--release",
                "--target",
                "wasm32-unknown-unknown",
            ])
            .output()
            .map(|output| -> Result<()> {
                std::io::stdout().write_all(&output.stdout).unwrap();
                std::io::stderr().write_all(&output.stderr).unwrap();

                if !output.status.success() {
                    Err(anyhow!("Failed to build wasm32-unknown-unknown!"))
                } else {
                    Ok(())
                }
            })??;

        // Install wasm bindgen
        println!("Opening wasm-api cargo.toml");
        let manifest_text = std::fs::read_to_string("./wasm-api/Cargo.toml")?;
        println!("Cargo.toml {:?}", manifest_text);
        let manifest = cargo_toml::Manifest::from_str(&manifest_text)?;

        let dep = manifest.dependencies.get("wasm-bindgen").unwrap();
        let version = dep.detail().unwrap().version.as_ref().unwrap().clone();

        let wasm_bindgen_executable = if cfg!(target_os = "windows") {
            String::from("./wasm-api/bin/wasm-bindgen.exe")
        } else {
            let curr_dir = std::env::current_dir().unwrap();
            String::from(curr_dir.join("wasm-api/bin/wasm-bindgen").to_str().unwrap())
        };

        println!(
            "Running wasm bindgen for version from {:?}",
            std::env::current_dir().unwrap()
        );
        let matches_version = Command::new(&wasm_bindgen_executable)
            .current_dir("wasm-api")
            .arg("--version")
            .output()
            .map(|output| {
                std::io::stdout().write_all(&output.stdout).unwrap();
                std::io::stderr().write_all(&output.stderr).unwrap();

                let desired = String::from("wasm-bindgen ") + &version;
                String::from_utf8(output.stdout)
                    .unwrap()
                    .trim_end_matches(&['\n', '\r'][..])
                    == desired
            })
            .unwrap_or(false);

        if !matches_version {
            println!("Install wasm bindgen");
            Command::new("cargo")
                .args([
                    "+nightly",
                    "install",
                    "-f",
                    "wasm-bindgen-cli",
                    "--version",
                    &version,
                    "--root",
                    "./wasm-api",
                ])
                .output()
                .map(|output| -> Result<()> {
                    std::io::stdout().write_all(&output.stdout).unwrap();
                    std::io::stderr().write_all(&output.stderr).unwrap();

                    if !output.status.success() {
                        Err(anyhow!("Failed to build wasm32-unknown-unknown!"))
                    } else {
                        Ok(())
                    }
                })??;
        }

        println!("Running wasm bindgen for packing");
        Command::new(&wasm_bindgen_executable)
            .current_dir("./wasm-api")
            .args([
                "--out-dir",
                ".",
                "../target/wasm32-unknown-unknown/release/twiggy_wasm_api.wasm",
            ])
            .output()
            .map(|output| -> Result<()> {
                std::io::stdout().write_all(&output.stdout).unwrap();
                std::io::stderr().write_all(&output.stderr).unwrap();

                if !output.status.success() {
                    Err(anyhow!("Failed to run wasm-bindgen"))
                } else {
                    Ok(())
                }
            })??;

        // This can fail and it's ok
        let _ = Command::new("cp")
            .current_dir("./wasm-api")
            .args(["twiggy_wasm_api_bg.wasm", "twiggy_wasm_api_bg2.wasm"])
            .output()
            .and_then(|_| {
                Command::new("wasm-opt")
                    .current_dir("./wasm-api")
                    .args([
                        "-Oz",
                        "-g",
                        "twiggy_wasm_api_bg2.wasm",
                        "-o",
                        "twiggy_wasm_api_bg.wasm",
                    ])
                    .output()
            });

        println!(
            "File size: {:?}",
            std::fs::metadata("./wasm-api/twiggy_wasm_api_bg.wasm")?.len()
        );
    }

    Ok(())
}
