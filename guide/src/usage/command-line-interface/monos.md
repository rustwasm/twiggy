# `twiggy monos`

The `twiggy monos` sub-command lists the generic function monomorphizations that
are contributing to code bloat.

```
 Apprx. Bloat Bytes │ Apprx. Bloat % │ Bytes │ %      │ Monomorphizations
────────────────────┼────────────────┼───────┼────────┼────────────────────────────────────────────────────────
               2141 ┊          3.68% ┊  3249 ┊  5.58% ┊ alloc::slice::merge_sort
                    ┊                ┊  1108 ┊  1.90% ┊     alloc::slice::merge_sort::hb3d195f9800bdad6
                    ┊                ┊  1108 ┊  1.90% ┊     alloc::slice::merge_sort::hfcf2318d7dc71d03
                    ┊                ┊  1033 ┊  1.77% ┊     alloc::slice::merge_sort::hcfca67f5c75a52ef
               1457 ┊          2.50% ┊  4223 ┊  7.26% ┊ <&'a T as core::fmt::Debug>::fmt
                    ┊                ┊  2766 ┊  4.75% ┊     <&'a T as core::fmt::Debug>::fmt::h1c27955d8de3ff17
                    ┊                ┊   636 ┊  1.09% ┊     <&'a T as core::fmt::Debug>::fmt::hea6a77c4dcddb7ac
                    ┊                ┊   481 ┊  0.83% ┊     <&'a T as core::fmt::Debug>::fmt::hfbacf6f5c9f53bb2
                    ┊                ┊   340 ┊  0.58% ┊     ... and 1 more.
               3759 ┊          6.46% ┊ 31160 ┊ 53.54% ┊ ... and 214 more.
               7357 ┊         12.64% ┊ 38632 ┊ 66.37% ┊ Σ [223 Total Rows]
```
