extern crate colored;
extern crate diff;

use std::process::Command;

use colored::Colorize;

use slurp;

test!(
    paths_test_called_once,
    "paths",
    "./fixtures/paths_test.wasm",
    "calledOnce"
);

test!(
    paths_test_called_twice,
    "paths",
    "./fixtures/paths_test.wasm",
    "calledTwice"
);

test!(
    paths_test_default_output,
    "paths",
    "./fixtures/paths_test.wasm"
);

test!(
    paths_test_default_output_desc,
    "paths",
    "./fixtures/paths_test.wasm",
    "--descending"
);

test!(
    paths_test_default_output_desc_with_depth,
    "paths",
    "./fixtures/paths_test.wasm",
    "--descending",
    "-d",
    "2"
);

test!(
    paths_wee_alloc,
    "paths",
    "./fixtures/wee_alloc.wasm",
    "wee_alloc::alloc_first_fit::h9a72de3af77ef93f",
    "hello",
    "goodbye"
);

test!(
    paths_wee_alloc_csv,
    "paths",
    "./fixtures/wee_alloc.wasm",
    "wee_alloc::alloc_first_fit::h9a72de3af77ef93f",
    "hello",
    "goodbye",
    "-f",
    "csv"
);

test!(
    paths_wee_alloc_with_depth_and_paths,
    "paths",
    "./fixtures/wee_alloc.wasm",
    "wee_alloc::alloc_first_fit::h9a72de3af77ef93f",
    "hello",
    "goodbye",
    "-d",
    "1",
    "-r",
    "2"
);

test!(
    paths_wee_alloc_json,
    "paths",
    "./fixtures/wee_alloc.wasm",
    "wee_alloc::alloc_first_fit::h9a72de3af77ef93f",
    "hello",
    "goodbye",
    "-d",
    "3",
    "-f",
    "json"
);

test!(
    paths_test_regex_called_any,
    "paths",
    "./fixtures/paths_test.wasm",
    "called.*",
    "--regex"
);

test!(
    paths_test_regex_exports,
    "paths",
    "./fixtures/paths_test.wasm",
    "export \".*\"",
    "--regex"
);

test!(
    paths_test_regex_exports_desc,
    "paths",
    "./fixtures/paths_test.wasm",
    "export \".*\"",
    "--descending",
    "--regex"
);

test!(
    issue_16,
    "paths",
    "./fixtures/mappings.wasm",
    "compute_column_spans"
);
