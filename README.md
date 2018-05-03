<meta charset="utf-8"/>

# `twiggy`!

[![](https://docs.rs/twiggy/badge.svg)](https://docs.rs/twiggy/)
[![](https://img.shields.io/crates/v/twiggy.svg)](https://crates.io/crates/twiggy)
[![](https://img.shields.io/crates/d/twiggy.svg)](https://crates.io/crates/twiggy)
[![Build Status](https://travis-ci.org/rustwasm/twiggy.svg?branch=master)](https://travis-ci.org/rustwasm/twiggy)

`twiggy` is a code size profiler.

It analyzes a binary's call graph to answer questions like:

* Why was this function included in the binary in the first place?

* What is the *retained size* of this function? I.e. how much space would be
  saved if I removed it and all the functions that become dead code after its
  removal.

Use `twiggy` to make your binaries slim!

![Twiggy](./twiggy.png)

--------------------------------------------------------------------------------

- [📦 Install](#-install)
- [💡 Concepts](#-concepts)
    - [Call Graph](#call-graph)
    - [Paths](#paths)
    - [Dominators and Retained Size](#dominators-and-retained-size)
    - [Generic Functions and Monomorphization](#generic-functions-and-monomorphization)
- [🏋️‍♀️ Usage](#%EF%B8%8F%EF%B8%8F-usage)
    - [⌨ Command Line Interface](#-command-line-interface)
        - [`twiggy top`](#twiggy-top)
        - [`twiggy paths`](#twiggy-paths)
        - [`twiggy monos`](#twiggy-monos)
        - [`twiggy dominators`](#twiggy-dominators)
    - [🦀 As a Crate](#-as-a-crate)
    - [🕸 On the Web with WebAssembly](#-on-the-web-with-webassembly)
- [🔎 Supported Binary Formats](#-supported-binary-formats)
- [🙌 Contributing](#-contributing)

## 📦 Install

Ensure that you have the [Rust toolchain installed](https://www.rust-lang.org/),
then run:

```
cargo install twiggy
```

## 💡 Concepts

### Call Graph

Consider the following functions:

```rust
pub fn shred() {
    gnar_gnar();
    bluebird();
}

fn gnar_gnar() {
    weather_report();
    pow();
}

fn bluebird() {
    weather_report();
}

fn weather_report() {
    shred();
}

fn pow() {
    fluffy();
    soft();
}

fn fluffy() {}

fn soft() {}

pub fn baker() {
    hood();
}

fn hood() {}
```

If we treat every function as a vertex in a graph, and if we add an edge from
*A* to *B* if function *A* calls function *B*, then we get the following *call
graph*:

[<img alt="Call Graph" src="./call-graph.svg"/>](./call-graph.svg)

### Paths

If there is a *path* where *A → B → ... → C* through the call graph, then we say
that *C* is *reachable* through from *A*. *Dead code* is code that is not
*reachable* in the call graph from any publicly exported functions (for
libraries) or the `main` function (for executables).

Imagine that `shred` from the last example was our executable's `main`
function. In this scenario, there is no path through the call graph from `shred`
to `baker` or `hood`, so they are dead code. We would expect that the linker
would remove them, and they wouldn't show up in the final binary.

But what if some function that you *thought* was dead code is appearing inside
your binary? Maybe it is deep down in some library you depend on, but inside a
submodule of that library that you aren't using, and you wouldn't expect it to
be included in the final binary.

In this scenario, `twiggy` can show you all the paths in the call graph that
lead to the unexpected function. This lets you understand why the unwelcome
function is present, and decide what you can do about it. Maybe if you
refactored your code to avoid calling *Y*, then there wouldn't be any paths to
the unwelcome function anymore, it would be dead code, and the linker would
remove it.

### Dominators and Retained Size

Imagine the `pow` function itself might is not very large. But it calls
functions `soft` and `fluffy`, both of which are **huge**. And they are both
*only* called by `pow`, so if `pow` were removed, then `soft` and `fluffy` would
both become dead code and get removed as well. Therefore, `pow`'s "real" size is
huge, even though it doesn't look like it at a glance. The *dominator*
relationship gives us a way to reason about the *retained size* of a function.

In a graph that is rooted at vertex *R*, vertex *A* is said to
[*dominate*][dominators] vertex *B* if every path in the graph from *R* to *B*
includes *A*. It follows that if *A* were removed from the graph, then *B* would
become unreachable.

In our call graphs, the roots are the `main` function (for executables) or
publicly exported functions (for libraries).

*V* is the *immediate dominator* of a vertex *U* if *V != U*, and there does not
exist another distinct vertex *W* that is dominated by *V* but also dominates
*U*. If we take all the vertices from a graph, remove the edges, and then add
edges for each immediate dominator relationship, then we get a tree. Here is the
dominator tree for our call graph from earlier, where `shred` is the root:

[<img alt="Dominator Tree" src="./dominator-tree.svg"/>](./dominator-tree.svg)

Using the dominator relationship, we can find the *retained size* of some
function by taking its shallow size and adding the retained sizes of each
function that it immediately dominates.

[dominators]: https://en.wikipedia.org/wiki/Dominator_(graph_theory)

### Generic Functions and Monomorphization

Generic functions with type parameters in Rust and template functions in C++ can
lead to code bloat if you aren't careful. Every time you instantiate these
generic functions with a concrete set of types, the compiler will *monomorphize*
the function, creating a copy of its body replacing its generic placeholders
with the specific operations that apply to the concrete types. This presents
many opportunities for compiler optimizations based on which particular concrete
types each copy of the function is working with, but these copies add up quickly
in terms of code size.

Example of monomorphization in Rust:

```rust
fn generic_function<T: MyTrait>(t: T) { ... }

// Each of these will generate a new copy of `generic_function`!
generic_function::<MyTraitImpl>(...);
generic_function::<AnotherMyTraitImpl>(...);
generic_function::<MyTraitImplAlso>(...);
```

Example of monomorphization in C++:

```c++
template<typename T>
void generic_function(T t) { ... }

// Each of these will also generate a new copy of `generic_function`!
generic_function<uint32_t>(...);
generic_function<bool>(...);
generic_function<MyClass>(...);
```

If you can afford the runtime cost of dynamic dispatch, then changing these
functions to use trait objects in Rust or virtual methods in C++ can likely save
a significant amounts of code size. With dynamic dispatch, the generic
function's body is not copied, and the generic bits within the function become
indirect function calls.

Example of dynamic dispatch in Rust:

```rust
fn generic_function(t: &MyTrait) { ... }
// or
fn generic_function(t: Box<MyTrait>) { ... }
// etc...

// No more code bloat!
let x = MyTraitImpl::new();
generic_function(&x);
let y = AnotherMyTraitImpl::new();
generic_function(&y);
let z = MyTraitImplAlso::new();
generic_function(&z);
```

Example of dynamic dispatch in C++:

```c++
class GenericBase {
  public:
    virtual void generic_impl() = 0;
};

class MyThing : public GenericBase {
  public
    virtual void generic_impl() override { ... }
};

class AnotherThing : public GenericBase {
  public
    virtual void generic_impl() override { ... }
};

class AlsoThing : public GenericBase {
  public
    virtual void generic_impl() override { ... }
};

void generic(GenericBase& thing) { ... }

// No more code bloat!
MyThing x;
generic(x);
AnotherThing y;
generic(y);
AlsoThing z;
generic(z);
```

`twiggy` can analyze a binary to find which generic functions are being
monomorphized repeatedly, and calculate an estimation of how much code size
could be saved by switching from monomorphization to dynamic dispatch.

## 🏋️‍♀️ Usage

### ⌨ Command Line Interface

`twiggy` is primarily a command line tool.

To get the most up-to-date usage for the version of `twiggy` that you've
installed, you can always run:

```
twiggy --help
```

Or, to get more information about a sub-command, run:

```
twiggy subcmd --help
```

#### `twiggy top`

The `twiggy top` sub-command summarizes and lists the top code size offenders in
a binary.

```
 Shallow Bytes │ Shallow % │ Item
───────────────┼───────────┼───────────────────────────────────────────────────────────────────────────────────────────────────
          1034 ┊    36.71% ┊ data[3]
           774 ┊    27.48% ┊ "function names" subsection
           225 ┊     7.99% ┊ wee_alloc::alloc_first_fit::h9a72de3af77ef93f
           164 ┊     5.82% ┊ hello
           152 ┊     5.40% ┊ wee_alloc::alloc_with_refill::hb32c1bbce9ebda8e
           136 ┊     4.83% ┊ <wee_alloc::size_classes::SizeClassAllocPolicy<'a> as wee_alloc::AllocPolicy>::new_cell_for_free_list::h3987e3054b8224e6
            76 ┊     2.70% ┊ <wee_alloc::LargeAllocPolicy as wee_alloc::AllocPolicy>::new_cell_for_free_list::h8f071b7bce0301ba
            44 ┊     1.56% ┊ goodbye
```

#### `twiggy paths`

The `twiggy paths` sub-command finds the call paths to a function in the given
binary's call graph. This tells you what other functions are calling this
function, why this function is not dead code, and therefore why it wasn't
removed by the linker.

```
 Shallow Bytes │ Shallow % │ Retaining Paths
───────────────┼───────────┼───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
           152 ┊     5.40% ┊ wee_alloc::alloc_with_refill::hb32c1bbce9ebda8e
               ┊           ┊   ⬑ func[2]
               ┊           ┊       ⬑ <wee_alloc::size_classes::SizeClassAllocPolicy<'a> as wee_alloc::AllocPolicy>::new_cell_for_free_list::h3987e3054b8224e6
               ┊           ┊           ⬑ func[5]
               ┊           ┊               ⬑ elem[0]
               ┊           ┊       ⬑ hello
               ┊           ┊           ⬑ func[8]
               ┊           ┊               ⬑ export "hello"
```

#### `twiggy monos`

The `twiggy monos` sub-command lists the generic function monomorphizations that
are contributing to code bloat.

```
 Apprx. Bloat Bytes │ Apprx. Bloat % │ Bytes │ %     │ Monomorphizations
────────────────────┼────────────────┼───────┼───────┼────────────────────────────────────────────────────────
               1977 ┊          3.40% ┊  3003 ┊ 5.16% ┊ alloc::slice::merge_sort
                    ┊                ┊  1026 ┊ 1.76% ┊     alloc::slice::merge_sort::hb3d195f9800bdad6
                    ┊                ┊  1026 ┊ 1.76% ┊     alloc::slice::merge_sort::hfcf2318d7dc71d03
                    ┊                ┊   951 ┊ 1.63% ┊     alloc::slice::merge_sort::hcfca67f5c75a52ef
               1302 ┊          2.24% ┊  3996 ┊ 6.87% ┊ <&'a T as core::fmt::Debug>::fmt
                    ┊                ┊  2694 ┊ 4.63% ┊     <&'a T as core::fmt::Debug>::fmt::h1c27955d8de3ff17
                    ┊                ┊   568 ┊ 0.98% ┊     <&'a T as core::fmt::Debug>::fmt::hea6a77c4dcddb7ac
                    ┊                ┊   433 ┊ 0.74% ┊     <&'a T as core::fmt::Debug>::fmt::hfbacf6f5c9f53bb2
                    ┊                ┊   301 ┊ 0.52% ┊     <&'a T as core::fmt::Debug>::fmt::h199e8e1c5752e6f1
```

#### `twiggy dominators`

The `twiggy dominators` sub-command displays the dominator tree of a binary's
call graph.

```
 Retained Bytes │ Retained % │ Dominator Tree
────────────────┼────────────┼────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
         284691 ┊     47.92% ┊ export "items_parse"
         284677 ┊     47.91% ┊   ⤷ func[17]
         284676 ┊     47.91% ┊       ⤷ items_parse
         128344 ┊     21.60% ┊           ⤷ func[47]
         128343 ┊     21.60% ┊               ⤷ twiggy_parser::wasm::<impl twiggy_parser::Parse<'a> for parity_wasm::elements::module::Module>::parse_items::h033e4aa1338b4363
          98403 ┊     16.56% ┊           ⤷ func[232]
          98402 ┊     16.56% ┊               ⤷ twiggy_ir::demangle::h7fb5cfffc912bc2f
          34206 ┊      5.76% ┊           ⤷ func[20]
          34205 ┊      5.76% ┊               ⤷ <parity_wasm::elements::section::Section as parity_wasm::elements::Deserialize>::deserialize::hdd814798147ca8dc
           2855 ┊      0.48% ┊           ⤷ func[552]
           2854 ┊      0.48% ┊               ⤷ <alloc::btree::map::BTreeMap<K, V>>::insert::he64f84697ccf122d
           1868 ┊      0.31% ┊           ⤷ func[53]
           1867 ┊      0.31% ┊               ⤷ twiggy_ir::ItemsBuilder::finish::h1b98f5cc4c80137d
```

### 🦀 As a Crate

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

For a more in-depth example, take a look at is the implementation of the
`twiggy` CLI crate.

### 🕸 On the Web with WebAssembly

First, ensure you have the `wasm32-unknown-unknown` Rust target installed and
up-to-date:

```
rustup install nightly
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

Next, install `wasm-bindgen`:

```
cargo +nightly install wasm-bindgen-cli
```

Finally, build `twiggy`'s WebAssembly API with `wasm-bindgen`:

```
cd twiggy/wasm-api
cargo +nightly build --release --target wasm32-unknown-unknown
wasm-bindgen ../target/wasm32-unknown-unknown/release/twiggy_wasm_api.wasm --out-dir .
```

This should produce two artifacts in the current directory:

1. `twiggy_wasm_api_bg.wasm`: The WebAssembly file containing `twiggy`.
2. `twiggy_wasm_api.js`: The JavaScript bindings to `twiggy`'s WebAssembly.

You can now use `twiggy` from JavaScript like this:

```js
import { Items, Monos } from './twiggy_wasm_api';

// Parse a binary's data into a collection of items.
const items = Items.parse(myData);

// Configure an analysis and its options.
const opts = Monos.new();
opts.set_max_generics(10);
opts.set_max_monos(10);

// Run the analysis on the parsed items.
const monos = JSON.parse(items.monos(opts));
```

## 🔎 Supported Binary Formats

`twiggy` currently supports these binary formats:

* ✔️ WebAssembly's `.wasm` format

`twiggy` doesn't support these binary formats (*yet!*):

* ❌ ELF
* ❌ Mach-O
* ❌ PE/COFF

Although `twiggy` doesn't currently support these binary formats, it is designed
with extensibility in mind. The input is translated into a format-agnostic
internal representation (IR), and adding support for new formats only requires
parsing them into this IR. The vast majority of `twiggy` will not need
modification.

**We would love to gain support for new binary formats, and if you're interested
in doing that implementation work,
[check out `CONTRIBUTING.md`!](./CONTRIBUTING.md)**

## 🙌 Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for hacking.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
