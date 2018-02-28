set -eux

case "$JOB" in
    "test")
        cargo test --all
        ;;

    "wasm")
        rustup target add wasm32-unknown-unknown

        cd analyze
        cargo build --release --target wasm32-unknown-unknown
        cd -

        cd ir
        cargo build --release --target wasm32-unknown-unknown
        cd -

        cd parser
        cargo build --release --target wasm32-unknown-unknown
        cd -

        cd traits
        cargo build --release --target wasm32-unknown-unknown
        cd -
        ;;

    *)
        echo "Error: unknown \$JOB = $JOB"
        exit 1
        ;;

esac
