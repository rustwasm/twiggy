# `twiggy diff`

The `twiggy diff` sub-command computes the delta size of each item between old
and new versions of a binary.

```
 Delta Bytes │ Item
─────────────┼──────────────────────────────────────────────
       -1034 ┊ data[3]
        -593 ┊ "function names" subsection
        +396 ┊ wee_alloc::alloc_first_fit::he2a4ddf96981c0ce
        +243 ┊ goodbye
        -226 ┊ wee_alloc::alloc_first_fit::h9a72de3af77ef93f
        -262 ┊ ... and 29 more.
       -1476 ┊ Σ [34 Total Rows]
```
