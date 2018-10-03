# `twiggy garbage`

The `twiggy garbage` sub-command finds and displays dead code and data that is
not transitively referenced by any exports or public functions.

```
$ twiggy garbage path/to/input.wasm
 Bytes │ Size % │ Garbage Item
───────┼────────┼──────────────────────
    11 ┊  5.58% ┊ unusedAddThreeNumbers
     8 ┊  4.06% ┊ unusedAddOne
     7 ┊  3.55% ┊ type[2]
     5 ┊  2.54% ┊ type[1]
     5 ┊  2.54% ┊ unusedChild
     4 ┊  2.03% ┊ type[0]
     1 ┊  0.51% ┊ func[0]
     1 ┊  0.51% ┊ func[1]
     1 ┊  0.51% ┊ func[2]
```
