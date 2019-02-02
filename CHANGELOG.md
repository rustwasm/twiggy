### 0.4.0

Released 2019-02-02.

* Add the ability to explicitly opt into using the experimental DWARF support.

* Summarize and hide potential false-positive garbage items.

* Fix a bug where non-C++ symbols were being demangled as C++ symbols
  incorrectly.

### 0.3.0

Released 2018/10/03.

* Twiggy has [a users guide](https://rustwasm.github.io/twiggy) now! [#170][]

* Added experimental, work-in-progress support for ELF and Mach-O binaries when
  they have DWARF debug info. [#74][]

* All subcommands default to displaying a maximum of 10 items at a time
  now. Additionally, they show a summary of the size and count of all the items
  that are not displayed. [#94][] [#98][] [#103][] [#113][]

* Added the `-a`/`--all` flag to `twiggy garbage` to display all garbage items
  without any max limit. [#118][]

* Added the `-a`/`--all`, `--all-generics`, and `--all-monos` flags to `twiggy
  monos` to list all generic functions, all monomorphizations of generic
  functions, and all of both generics and their monomorphizations. [#120][]

* Added support for using regexes to find the difference in particular function
  sizes with `twiggy diff --regex`. [#129][]

* Fixed a bug where wasm table elements referencing imported functions would
  cause integer underflow. [#151][]

* Consider wasm tables roots in the graph, and make edges table -> element,
  rather than element -> table. The latter is because a table logically owns its
  elements, not the other way around. The former is because dynamic virtual
  calls are not statically analyzable, so we have to consider all virtual
  functions (aka function table elements) as psuedo-roots in the graph. These
  two changes allow us to see when the table is heavy in the dominator tree
  because a bunch of dynamic indirect calls that may or may not be possible at
  run time are entrained in the function table because the compiler/linker
  couldn't statically prove that they won't happen. [#153][]

[#74]: https://github.com/rustwasm/twiggy/pull/74
[#94]: https://github.com/rustwasm/twiggy/pull/94
[#98]: https://github.com/rustwasm/twiggy/pull/98
[#103]: https://github.com/rustwasm/twiggy/pull/103
[#113]: https://github.com/rustwasm/twiggy/pull/113
[#118]: https://github.com/rustwasm/twiggy/pull/118
[#120]: https://github.com/rustwasm/twiggy/pull/120
[#129]: https://github.com/rustwasm/twiggy/pull/129
[#151]: https://github.com/rustwasm/twiggy/pull/151
[#153]: https://github.com/rustwasm/twiggy/pull/153
[#170]: https://github.com/rustwasm/twiggy/pull/170

### 0.2.0

Released 2018/06/25.

* Added [@data-pup][] to the Twiggy team! \o/

* Added the `twiggy diff` subcommand to compare two versions of the same
  binary. [#49][] [#12][]

* Added the `twiggy garbage` subcommand to list code and data that is not
  transitively referenced by any exports / public functions. [#48][] [#50][]

* Added the ability to emit results as CSV. Pass the `--format csv` flags. [#44][]

* `twiggy paths` will now default to printing the paths to all items if no
  specific item is given as an argument. [#57][] [#63][]

* Added a `--regex` option to `twiggy paths` and `twiggy dominators`. This
  allows you to filter items by regexp, for example to only list items matching
  `std::.*`. [#58][] [#65][] [#59][] [#68][]

[#49]: https://github.com/rustwasm/twiggy/pull/49
[#12]: https://github.com/rustwasm/twiggy/issues/12
[#50]: https://github.com/rustwasm/twiggy/pull/50
[#48]: https://github.com/rustwasm/twiggy/issues/48
[#57]: https://github.com/rustwasm/twiggy/issues/57
[#63]: https://github.com/rustwasm/twiggy/pull/63
[#44]: https://github.com/rustwasm/twiggy/pull/44
[#65]: https://github.com/rustwasm/twiggy/pull/65
[#58]: https://github.com/rustwasm/twiggy/issues/58
[#58]: https://github.com/rustwasm/twiggy/issues/59
[#68]: https://github.com/rustwasm/twiggy/pull/68
[@data-pup]: https://github.com/data-pup

### 0.1.0

Released 2018/05/03.

* Initial release!
