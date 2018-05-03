#!/usr/bin/env bash

set -eux

cd "$(dirname $0)"

for dir in ir traits parser opt analyze twiggy; do
    cd "$dir"
    echo cargo publish
    cd -
done
