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

        # Install wasm-bindgen at the correct version, if necessary.
        test -x ./bin/wasm-bindgen \
            && test "$(./bin/wasm-bindgen --version | xargs)" == "wasm-bindgen 0.2.12" \
                || cargo +nightly install -f wasm-bindgen-cli --version 0.2.12 --root "$(pwd)"

        ./bin/wasm-bindgen --out-dir . ../target/wasm32-unknown-unknown/release/twiggy_wasm_api.wasm

        if [[ $(which wasm-opt) != "" ]]; then
            temp=$(mktemp "twiggy-XXXXXX.wasm")
            cp twiggy_wasm_api_bg.wasm "$temp"
            wasm-opt -Oz -g "$temp" -o twiggy_wasm_api_bg.wasm
        fi

        wc -c twiggy_wasm_api_bg.wasm
        ;;

    *)
        echo "Error: unknown \$JOB = $JOB"
        exit 1
        ;;

esac
