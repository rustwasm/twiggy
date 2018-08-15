#!/usr/bin/env bash

set -eux

ROOT="$(dirname "$0")/.."
cd "$ROOT"

function main {
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
            local version="$(get_wasm_bindgen_version)"
            test -x ./bin/wasm-bindgen \
                && test "$(./bin/wasm-bindgen --version | xargs)" == "wasm-bindgen $version" \
                    || cargo +nightly install -f wasm-bindgen-cli --version "$version" --root "$(pwd)"

            ./bin/wasm-bindgen --out-dir . ../target/wasm32-unknown-unknown/release/twiggy_wasm_api.wasm

            if [[ $(which wasm-opt) != "" ]]; then
                local temp=$(mktemp "twiggy-XXXXXX.wasm")
                cp twiggy_wasm_api_bg.wasm "$temp"
                wasm-opt -Oz -g "$temp" -o twiggy_wasm_api_bg.wasm
                rm "$temp"
            fi

            wc -c twiggy_wasm_api_bg.wasm
            ;;

        *)
            echo "Error: unknown \$JOB = $JOB"
            exit 1
            ;;
    esac
}

function get_wasm_bindgen_version {
    grep wasm-bindgen -A 1 Cargo.toml \
        | grep version \
        | cut -f2 -d '=' \
        | tr '"' ' ' \
        | xargs
}

main
