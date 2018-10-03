# `twiggy monos`

The `twiggy monos` sub-command lists the generic function monomorphizations that
are contributing to code bloat.

```
$ twiggy monos path/to/input.wasm
 Apprx. Bloat Bytes │ Apprx. Bloat % │ Bytes │ %     │ Monomorphizations
────────────────────┼────────────────┼───────┼───────┼────────────────────────────────────────────────────────
               1977 ┊          3.40% ┊  3003 ┊ 5.16% ┊ alloc::slice::merge_sort
                    ┊                ┊  1026 ┊ 1.76% ┊     alloc::slice::merge_sort::hb3d195f9800bdad6
                    ┊                ┊  1026 ┊ 1.76% ┊     alloc::slice::merge_sort::hfcf2318d7dc71d03
                    ┊                ┊   951 ┊ 1.63% ┊     alloc::slice::merge_sort::hcfca67f5c75a52ef
               1302 ┊          2.24% ┊  3996 ┊ 6.87% ┊ <&'a T as core::fmt::Debug>::fmt
                    ┊                ┊  2694 ┊ 4.63% ┊     <&'a T as core::fmt::Debug>::fmt::h1c27955d8de3ff17
                    ┊                ┊   568 ┊ 0.98% ┊     <&'a T as core::fmt::Debug>::fmt::hea6a77c4dcddb7ac
                    ┊                ┊   433 ┊ 0.74% ┊     <&'a T as core::fmt::Debug>::fmt::hfbacf6f5c9f53bb2
                    ┊                ┊   301 ┊ 0.52% ┊     <&'a T as core::fmt::Debug>::fmt::h199e8e1c5752e6f1
```
