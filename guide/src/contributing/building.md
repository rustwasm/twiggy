# Building

## Building for the Native Target

```
$ cargo build --all --exclude twiggy-wasm-api
```

## Building for the `wasm32-unknown-unknown` Target

```
$ JOB=wasm ./ci/script.sh
```
