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
