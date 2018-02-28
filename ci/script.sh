set -eux

case "$JOB" in
    "test")
        cargo test --all
        ;;

    "wasm")
        rustup target add wasm32-unknown-unknown

        cd wasm-api
        cargo build --release --target wasm32-unknown-unknown
        ;;

    *)
        echo "Error: unknown \$JOB = $JOB"
        exit 1
        ;;

esac
