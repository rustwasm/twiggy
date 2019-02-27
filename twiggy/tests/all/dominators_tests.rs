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
    dominators_json_prints_multiple_root_items,
    "dominators",
    "./fixtures/paths_test.wasm",
    "-f",
    "json",
    "--regex",
    "called.*"
);

test!(
    dominators_regex_any_func,
    "dominators",
    "./fixtures/paths_test.wasm",
    "--regex",
    "func\\[[0-9]+\\]"
);

test!(
    dominators_csv_does_not_summarize_garbage_if_all_items_are_reachable,
    "dominators",
    "./fixtures/paths_test.wasm",
    "-f",
    "csv"
);

test!(
    dominators_summarizes_unreachable_items,
    "dominators",
    "./fixtures/garbage.wasm",
    "-d",
    "1"
);
