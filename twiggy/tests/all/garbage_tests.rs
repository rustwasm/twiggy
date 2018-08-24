extern crate colored;
extern crate diff;

use std::process::Command;

use colored::Colorize;

use slurp;

test!(garbage, "garbage", "./fixtures/garbage.wasm");

test!(
    garbage_top_2,
    "garbage",
    "./fixtures/garbage.wasm",
    "-n",
    "2"
);

test!(
    garbage_json,
    "garbage",
    "./fixtures/garbage.wasm",
    "-f",
    "json"
);

test!(
    garbage_top_2_json,
    "garbage",
    "./fixtures/garbage.wasm",
    "-f",
    "json",
    "-n",
    "2"
);

test!(
    garbage_wee_alloc_top_10,
    "garbage",
    "./fixtures/wee_alloc.wasm"
);

test!(
    garbage_wee_alloc_all,
    "garbage",
    "./fixtures/wee_alloc.wasm",
    "-a"
);

test!(
    garbage_wee_alloc_top_10_json,
    "garbage",
    "./fixtures/wee_alloc.wasm",
    "-f",
    "json"
);

test!(
    garbage_wee_alloc_all_json,
    "garbage",
    "./fixtures/wee_alloc.wasm",
    "-f",
    "json",
    "-a"
);
