#!/usr/bin/env bash

set -eux

cd "$(dirname "$0")/.."

case "$JOB" in
    "test")
        cargo test --all --exclude twiggy-wasm-api
        ;;

    "wasm")
        rustup update nightly
        rustup target add wasm32-unknown-unknown --toolchain nightly

        cd ./wasm-api
        cargo +nightly build --release --target wasm32-unknown-unknown

        test -x ./bin/wasm-bindgen || cargo install wasm-bindgen-cli --version 0.2.3 --root "$(pwd)"
        ./bin/wasm-bindgen --out-dir . ../target/wasm32-unknown-unknown/release/twiggy_wasm_api.wasm
        wc -c *.wasm
        ;;

    *)
        echo "Error: unknown \$JOB = $JOB"
        exit 1
        ;;

esac
