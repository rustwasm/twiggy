# `twiggy paths`

The `twiggy paths` sub-command finds the call paths to a function in the given
binary's call graph. This tells you what other functions are calling this
function, why this function is not dead code, and therefore why it wasn't
removed by the linker.

```
 Shallow Bytes │ Shallow % │ Retaining Paths
───────────────┼───────────┼───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
           153 ┊     5.43% ┊ wee_alloc::alloc_with_refill::hb32c1bbce9ebda8e
               ┊           ┊   ⬑ <wee_alloc::size_classes::SizeClassAllocPolicy<'a> as wee_alloc::AllocPolicy>::new_cell_for_free_list::h3987e3054b8224e6
               ┊           ┊       ⬑ elem[0]
               ┊           ┊           ⬑ table[0]
               ┊           ┊   ⬑ hello
               ┊           ┊       ⬑ export "hello"

```
