#!/usr/bin/env bash

set -eux

cd "$(dirname $0)"

for dir in ir traits parser opt analyze twiggy; do
    cd "$dir"

    if [[ "$dir" == "opt" || "$dir" == "analyze" ]]; then
        cargo publish --no-verify
    else
        cargo publish
    fi

    cd -
done
