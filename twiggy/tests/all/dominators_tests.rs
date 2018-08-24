extern crate colored;
extern crate diff;

use std::process::Command;

use colored::Colorize;

use slurp;

test!(
    dominators_wee_alloc,
    "dominators",
    "./fixtures/wee_alloc.wasm"
);

test!(
    dominators_wee_alloc_json,
    "dominators",
    "./fixtures/wee_alloc.wasm",
    "-f",
    "json"
);

test!(
    dominators_wee_alloc_csv,
    "dominators",
    "./fixtures/wee_alloc.wasm",
    "-f",
    "csv"
);

test!(
    dominators_wee_alloc_with_depth_and_row,
    "dominators",
    "./fixtures/wee_alloc.wasm",
    "-d",
    "5",
    "-r",
    "3"
);

test!(
    dominators_wee_alloc_subtree,
    "dominators",
    "./fixtures/wee_alloc.wasm",
    "hello"
);

test!(
    dominators_wee_alloc_subtree_json,
    "dominators",
    "./fixtures/wee_alloc.wasm",
    "-f",
    "json",
    "hello"
);

test!(
    dominators_regex_any_func,
    "dominators",
    "./fixtures/paths_test.wasm",
    "--regex",
    "func\\[[0-9]+\\]"
);
