test!(
    diff_wee_alloc,
    "diff",
    "./fixtures/wee_alloc.wasm",
    "./fixtures/wee_alloc.2.wasm"
);

test!(
    diff_wee_alloc_top_5,
    "diff",
    "./fixtures/wee_alloc.wasm",
    "./fixtures/wee_alloc.2.wasm",
    "-n",
    "5"
);

// TODO: Update this test once `--all` flag is added.
test!(
    diff_wee_alloc_all,
    "diff",
    "./fixtures/wee_alloc.wasm",
    "./fixtures/wee_alloc.2.wasm",
    "-n",
    "100"
);

test!(
    diff_wee_alloc_json,
    "diff",
    "./fixtures/wee_alloc.wasm",
    "./fixtures/wee_alloc.2.wasm",
    "-f",
    "json"
);

test!(
    diff_wee_alloc_json_top_5,
    "diff",
    "./fixtures/wee_alloc.wasm",
    "./fixtures/wee_alloc.2.wasm",
    "-f",
    "json",
    "-n",
    "5"
);

test!(
    diff_wee_alloc_csv,
    "diff",
    "./fixtures/wee_alloc.wasm",
    "./fixtures/wee_alloc.2.wasm",
    "-f",
    "csv"
);

test!(
    diff_wee_alloc_csv_top_5,
    "diff",
    "./fixtures/wee_alloc.wasm",
    "./fixtures/wee_alloc.2.wasm",
    "-f",
    "csv",
    "-n",
    "5"
);

test!(
    diff_test_regex_wee_alloc,
    "diff",
    "./fixtures/wee_alloc.wasm",
    "./fixtures/wee_alloc.2.wasm",
    "--regex",
    "(data|type)\\[\\d*\\]"
);

test!(
    diff_test_exact_wee_alloc,
    "diff",
    "./fixtures/wee_alloc.wasm",
    "./fixtures/wee_alloc.2.wasm",
    "hello",
    "goodbye"
);
