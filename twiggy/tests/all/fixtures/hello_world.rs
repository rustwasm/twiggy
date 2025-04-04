//! Rebuild `hello_elf` with:
//!
//! ```
//! rustc --target x86_64-unknown-linux-gnu hello_world.rs -o hello_elf -C lto=fat -C opt-level=z -Clink-arg=-Wl,--emit-relocs
//! ```
//!
//! Rebuild `hello_mach` with:
//!
//! ```
//! rustc +nightly --target x86_64-apple-darwin hello_world.rs -o hello_mach.o -C lto=fat -C opt-level=z
//! ```
//! NOTE: The above is not working for me on Ubuntu. This causes an error when `ld` is invoked.

fn main() {
    println!("Hello, world!");
}
