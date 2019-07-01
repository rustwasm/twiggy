# 🦀 As a Crate

`twiggy` is divided into a collection of crates that you can use
programmatically, but no long-term stability is promised. We will follow semver
as best as we can, but will err on the side of being more conservative with
breaking version bumps than might be strictly necessary.

Here is a simple example:

```rust
extern crate twiggy_analyze;
extern crate twiggy_opt;
extern crate twiggy_parser;

use std::fs;
use std::io;

fn main() {
    let mut file = fs::File::open("path/to/some/binary").unwrap();
    let mut data = vec![];
    file.read_to_end(&mut data).unwrap();

    let items = twiggy_parser::parse(&data).unwrap();

    let options = twiggy_opt::Top::default();
    let top = twiggy_analyze::top(&mut items, &options).unwrap();

    let mut stdout = io::stdout();
    top.emit_text(&items, &mut stdout).unwrap();
}
```

For a more in-depth example, take a look at the implementation of the
`twiggy` CLI crate.
