# 🕸 On the Web with WebAssembly

First, ensure you have the `wasm32-unknown-unknown` Rust target installed and
up-to-date:

```
rustup target add wasm32-unknown-unknown
```

Next, install `wasm-bindgen`:

```
cargo install wasm-bindgen-cli
```

Finally, build `twiggy`'s WebAssembly API with `wasm-bindgen`:

```
cd twiggy/wasm-api
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen ../target/wasm32-unknown-unknown/release/twiggy_wasm_api.wasm --out-dir .
```

This should produce two artifacts in the current directory:

1. `twiggy_wasm_api_bg.wasm`: The WebAssembly file containing `twiggy`.
2. `twiggy_wasm_api.js`: The JavaScript bindings to `twiggy`'s WebAssembly.

You can now use `twiggy` from JavaScript like this:

```js
import { Items, Monos } from './twiggy_wasm_api';

// Parse a binary's data into a collection of items.
const items = Items.parse(myData);

// Configure an analysis and its options.
const opts = Monos.new();
opts.set_max_generics(10);
opts.set_max_monos(10);

// Run the analysis on the parsed items.
const monos = JSON.parse(items.monos(opts));
```
