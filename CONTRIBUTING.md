# Contributing to `svelte`

Hi! We'd love to have your contributions! If you want help or mentorship, reach
out to us in a GitHub issue, or ping `fitzgen` in [`#rust-wasm` on
`irc.mozilla.org`](irc://irc.mozilla.org#rust-wasm) and introduce yourself.

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->


- [Code of Conduct](#code-of-conduct)
- [Building and Testing](#building-and-testing)
  - [Building](#building)
  - [Testing](#testing)
    - [Authoring New Tests](#authoring-new-tests)
- [Automatic Code Formatting](#automatic-code-formatting)
- [Pull Requests](#pull-requests)
- [Contributions We Want](#contributions-we-want)
- [Team](#team)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Code of Conduct

We abide by the [Rust Code of Conduct][coc] and ask that you do as well.

[coc]: https://www.rust-lang.org/en-US/conduct.html

## Building and Testing

### Building

```
$ cargo build --all
```

### Testing

```
$ cargo test --all
```

#### Authoring New Tests

Integration tests live in the `svelte/tests` directory:

```
svelte/tests
├── expectations
├── fixtures
└── tests.rs
```

* The `svelte/tests/tests.rs` file contains the `#[test]` definitions.

* The `svelte/tests/fixtures` directory contains input binaries for tests.

* The `svelte/tests/expectations` directory contains the expected output of test
  commands.

## Automatic Code Formatting

We use [`rustfmt`](https://github.com/rust-lang-nursery/rustfmt) to enforce a
consistent code style across the whole code base.

You can install the latest version of `rustfmt` with this command:

```
$ rustup update
$ rustup component add rustfmt-preview
```

Ensure that `~/.rustup/toolchains/$YOUR_HOST_TARGET/bin/` is on your `$PATH`.

Once that is taken care of, you can (re)format all code by running this command
from the root of the repository:

```
$ cargo fmt --all
```

## Pull Requests

All pull requests must be reviewed and approved of by at least one [team](#team)
member before merging. See [Contributions We Want](#contributions-we-want) for
details on what should be included in what kind of pull request.

## Contributions We Want

* **Bug fixes!** Include a regression test.

* **Support for more binary formats!** See [this issue][more-formats] for
  details.

* **New analyses and queries!** Help expose information about monomorphizations
  or inlining. Report diffs between two versions of the same binary. Etc...

If you make two of these kinds of contributions, you should seriously consider
joining our [team](#team)!

Where we need help:

* Issues labeled ["help wanted"][help-wanted] are issues where we could use a
  little help from you.

* Issues labeled ["mentored"][mentored] are issues that don't really involve any
  more investigation, just implementation. We've outlined what needs to be done,
  and a [team](#team) member has volunteered to help whoever claims the issue to
  implement it, and get the implementation merged.

* Issues labeled ["good first issue"][gfi] are issues where fixing them would be
  a great introduction to the code base.

[more-formats]: https://github.com/fitzgen/svelte/issues/4
[help-wanted]: https://github.com/fitzgen/svelte/labels/help%20wanted
[mentored]: https://github.com/fitzgen/svelte/labels/mentored
[gfi]: https://github.com/fitzgen/svelte/labels/good%20first%20issue

## Team

| [<img alt="fitzgen" src="https://avatars2.githubusercontent.com/u/74571?s=117&v=4" width="117">](https://github.com/fitzgen) |
|:---:|
| [`fitzgen`](https://github.com/fitzgen) |

Larger, more nuanced decisions about design, architecture, breaking changes,
trade offs, etc are made by team consensus. In other words, decisions on things
that aren't straightforward improvements or bug fixes to things that already
exist in `svelte`. If consensus can't be made, then `fitzgen` has the last
word.

**We need more team members!**
[Drop a comment on this issue if you are interested in joining.][join]

[join]: https://github.com/fitzgen/svelte/issues/3
