# `twiggy diff`

The `twiggy diff` sub-command computes the delta size of each item between old
and new versions of a binary.

```
$ twiggy diff path/to/old.wasm path/to/new.wasm
 Delta Bytes │ Item
─────────────┼────────────────────────────────────────────────────────────────────
       -1476 ┊ <total>
       -1034 ┊ data[3]
        -593 ┊ "function names" subsection
        +395 ┊ wee_alloc::alloc_first_fit::he2a4ddf96981c0ce
        +243 ┊ goodbye
        -225 ┊ wee_alloc::alloc_first_fit::h9a72de3af77ef93f
        -152 ┊ wee_alloc::alloc_with_refill::hb32c1bbce9ebda8e
        +145 ┊ <wee_alloc::neighbors::Neighbors<'a, T>>::remove::hc9e5d4284e8233b8
```
