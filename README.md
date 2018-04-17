# `twiggy`!
!
[![](https://docs.rs/twiggy/badge.svg)](https://docs.rs/twiggy/)!
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

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->


- [Install](#install)
- [Usage](#usage)
- [Concepts](#concepts)
  - [Call Graph](#call-graph)
  - [Paths](#paths)
  - [Dominators and Retained Size](#dominators-and-retained-size)
- [Supported Binary Formats](#supported-binary-formats)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Install

Ensure that you have the [Rust toolchain installed](https://www.rust-lang.org/),
then run:

```
$ cargo install --git https://github.com/rustwasm/twiggy.git
```

## Usage

```
$ twiggy --help
```

## Concepts

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

You can use the `twiggy paths` subcommand to view the paths to a function in a
given binary's call graph:

```
$ twiggy paths wee_alloc.wasm 'wee_alloc::alloc_first_fit::h9a72de3af77ef93f'
 Shallow Bytes │ Shallow % │ Retaining Paths
───────────────┼───────────┼───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
           225 ┊     7.99% ┊ wee_alloc::alloc_first_fit::h9a72de3af77ef93f
               ┊           ┊   ⬑ func[3]
               ┊           ┊       ⬑ wee_alloc::alloc_with_refill::hb32c1bbce9ebda8e
               ┊           ┊           ⬑ func[2]
               ┊           ┊               ⬑ <wee_alloc::size_classes::SizeClassAllocPolicy<'a> as wee_alloc::AllocPolicy>::new_cell_for_free_list::h3987e3054b8224e6
               ┊           ┊                   ⬑ func[5]
               ┊           ┊                       ⬑ elem[0]
               ┊           ┊               ⬑ hello
               ┊           ┊                   ⬑ func[8]
               ┊           ┊                       ⬑ export "hello"
```

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

You can use the `twiggy dominators` subcommand to view the dominator tree for a
given binary's call graph:

```
$ twiggy dominators wee_alloc.wasm
 Retained Bytes │ Retained % │ Dominator Tree
────────────────┼────────────┼────────────────────────────────────────────────────────────────────────
            774 ┊     27.48% ┊ "function names" subsection
            564 ┊     20.02% ┊ export "hello"
            556 ┊     19.74% ┊   ⤷ func[8]
            551 ┊     19.56% ┊       ⤷ hello
            387 ┊     13.74% ┊           ⤷ func[2]
            378 ┊     13.42% ┊               ⤷ wee_alloc::alloc_with_refill::hb32c1bbce9ebda8e
            226 ┊      8.02% ┊                   ⤷ func[3]
            225 ┊      7.99% ┊                       ⤷ wee_alloc::alloc_first_fit::h9a72de3af77ef93f
              8 ┊      0.28% ┊               ⤷ type[4]
              4 ┊      0.14% ┊       ⤷ type[5]
             59 ┊      2.09% ┊ export "goodbye"
             49 ┊      1.74% ┊   ⤷ func[9]
             44 ┊      1.56% ┊       ⤷ goodbye
              4 ┊      0.14% ┊       ⤷ type[3]
             11 ┊      0.39% ┊ export "memory"
              2 ┊      0.07% ┊   ⤷ memory[0]
```

[dominators]: https://en.wikipedia.org/wiki/Dominator_(graph_theory)

## Supported Binary Formats

* WebAssembly's `.wasm` format

Although `twiggy` doesn't currently support ELF, Mach-O, or PE/COFF, it is
designed with extensibility in mind. The input is translated into a
format-agnostic internal representation (IR), and adding support for new formats
only requires parsing them into this IR. The vast majority of `twiggy` will not
need modification.

We would love to gain support for new binary formats, and if you're interested
in doing that implementation work, [check out
`CONTRIBUTING.md`](./CONTRIBUTING.md).
