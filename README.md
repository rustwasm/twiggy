# `svelte`

[![](https://docs.rs/svelte/badge.svg)](https://docs.rs/svelte/)
[![](https://img.shields.io/crates/v/svelte.svg)](https://crates.io/crates/svelte)
[![](https://img.shields.io/crates/d/svelte.svg)](https://crates.io/crates/svelte)
[![Build Status](https://travis-ci.org/fitzgen/svelte.svg?branch=master)](https://travis-ci.org/fitzgen/svelte)

`svelte` is a code size profiler.

It analyzes a binary's call graph to answer questions like:

* Why was this function included in the binary in the first place?

* What is the *retained size* of this function? I.e. how much space would be
  saved if I removed it and all the functions that become dead code after its
  removal.

Use `svelte` to make your binaries slim!

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
$ cargo install svelte
```

## Usage

```
$ svelte --help
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

In this scenario, `svelte` can show you all the paths in the call graph that
lead to the unexpected function. This lets you understand why the unwelcome
function is present, and decide what you can do about it. Maybe if you
refactored your code to avoid calling *Y*, then there wouldn't be any paths to
the unwelcome function anymore, it would be dead code, and the linker would
remove it.

You can use the `svelte paths` subcommand to view the paths to a function in a
given binary's call graph.

### Dominators and Retained Size

A function *F* might not be very large. But it might call functions *G* and *H*,
both of which are huge. And they are *only* called by *F*, so if *F* were
removed, then *G* and *H* would both become dead code and get removed as
well. Therefore, *F*'s "real" size is huge, even though it doesn't look like it
on paper. The *dominator* relationship gives us a way to reason about the
*retained size* of a function.

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

You can use the `svelte dominators` subcommand to view the dominator tree for a
given binary's call graph.

[dominators]: https://en.wikipedia.org/wiki/Dominator_(graph_theory)

## Supported Binary Formats

* WebAssembly's `.wasm` format

Although `svelte` doesn't currently support ELF, Mach-O, or PE/COFF, it is
designed with extensibility in mind. The input is translated into a
format-agnostic internal representation (IR), and adding support for new formats
only requires parsing them into this IR. The vast majority of `svelte` will not
need modification.

We would love to gain support for new binary formats, and if you're interested
in doing that implementation work, [check out
`CONTRIBUTING.md`](./CONTRIBUTING.md).
