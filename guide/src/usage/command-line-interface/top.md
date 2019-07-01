# `twiggy top`

The `twiggy top` sub-command summarizes and lists the top code size offenders in
a binary.

```
 Shallow Bytes │ Shallow % │ Item
───────────────┼───────────┼─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
          1034 ┊    36.71% ┊ data[3]
           777 ┊    27.58% ┊ "function names" subsection
           226 ┊     8.02% ┊ wee_alloc::alloc_first_fit::h9a72de3af77ef93f
           165 ┊     5.86% ┊ hello
           153 ┊     5.43% ┊ wee_alloc::alloc_with_refill::hb32c1bbce9ebda8e
           137 ┊     4.86% ┊ <wee_alloc::size_classes::SizeClassAllocPolicy<'a> as wee_alloc::AllocPolicy>::new_cell_for_free_list::h3987e3054b8224e6
            77 ┊     2.73% ┊ <wee_alloc::LargeAllocPolicy as wee_alloc::AllocPolicy>::new_cell_for_free_list::h8f071b7bce0301ba
            45 ┊     1.60% ┊ goodbye
            25 ┊     0.89% ┊ data[1]
            25 ┊     0.89% ┊ data[2]
           153 ┊     5.43% ┊ ... and 27 more.
          2817 ┊   100.00% ┊ Σ [37 Total Rows]
```
