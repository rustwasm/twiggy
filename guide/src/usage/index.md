# 🏋️‍♀️ Usage

Twiggy is primarily a command line tool, but it can also be used as a library
crate from within other Rust projects, or compiled to WebAssembly and used from
JavaScript on the Web or from Node.js


## 🏋️‍♀️ wasm-pack

In order to get usable output with `wasm-pack`, we need debug info in the resulting `.wasm` file.

Add this to your Cargo.toml
```
[package.metadata.wasm-pack.profile.release]
wasm-opt = ['-g', '-O']
```
