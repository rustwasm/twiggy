# 🔎 Supported Binary Formats

## Full Support

`twiggy` currently supports these binary formats:

* ✔️ WebAssembly's `.wasm` format

## Partial, Work-in-Progress Support

`twiggy` has partial, work-in-progress support for these binary formats *when
they have [DWARF][dwarf] debug info*:

* ⚠ ELF
* ⚠ Mach-O

## Unsupported

* ❌ PE/COFF

Although `twiggy` doesn't currently support these binary formats, it is designed
with extensibility in mind. The input is translated into a format-agnostic
internal representation (IR), and adding support for new formats only requires
parsing them into this IR. The vast majority of `twiggy` will not need
modification.

We would love to gain support for new binary formats! If you're interested in
helping out with that implementation work, [read this to learn how to contribute
to Twiggy!](./contributing/index.html)

[dwarf]: http://dwarfstd.org/
