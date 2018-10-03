# `twiggy paths`

The `twiggy paths` sub-command finds the call paths to a function in the given
binary's call graph. This tells you what other functions are calling this
function, why this function is not dead code, and therefore why it wasn't
removed by the linker.

```
$ twiggy paths path/to/wee_alloc.wasm 'wee_alloc::alloc_with_refill::hb32c1bbce9ebda8e'
 Shallow Bytes │ Shallow % │ Retaining Paths
───────────────┼───────────┼───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
           152 ┊     5.40% ┊ wee_alloc::alloc_with_refill::hb32c1bbce9ebda8e
               ┊           ┊   ⬑ func[2]
               ┊           ┊       ⬑ <wee_alloc::size_classes::SizeClassAllocPolicy<'a> as wee_alloc::AllocPolicy>::new_cell_for_free_list::h3987e3054b8224e6
               ┊           ┊           ⬑ func[5]
               ┊           ┊               ⬑ elem[0]
               ┊           ┊       ⬑ hello
               ┊           ┊           ⬑ func[8]
               ┊           ┊               ⬑ export "hello"
```
