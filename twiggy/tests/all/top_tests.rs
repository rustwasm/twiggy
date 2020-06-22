test!(top_mappings, "top", "-n", "10", "./fixtures/mappings.wasm");

test!(
    top_wee_alloc,
    "top",
    "-n",
    "10",
    "./fixtures/wee_alloc.wasm"
);

test!(
    top_retained_wee_alloc,
    "top",
    "-n",
    "10",
    "--retained",
    "./fixtures/wee_alloc.wasm"
);

test!(
    top_retained_mappings,
    "top",
    "-n",
    "10",
    "--retained",
    "./fixtures/mappings.wasm"
);

test!(
    top_2_json,
    "top",
    "./fixtures/wee_alloc.wasm",
    "-n",
    "2",
    "-f",
    "json"
);

test!(
    top_2_json_retained,
    "top",
    "./fixtures/wee_alloc.wasm",
    "--retained",
    "-n",
    "2",
    "-f",
    "json"
);

test!(
    top_2_csv,
    "top",
    "./fixtures/wee_alloc.wasm",
    "-n",
    "4",
    "-f",
    "csv"
);

test!(
    top_2_csv_retained,
    "top",
    "./fixtures/wee_alloc.wasm",
    "--retained",
    "-n",
    "4",
    "-f",
    "csv"
);

// This should not fail to open and write `whatever-output.txt`.
test!(
    output_to_file,
    "top",
    "./fixtures/wee_alloc.wasm",
    "-o",
    "whatever-output.txt"
);

// Regression test for https://github.com/rustwasm/twiggy/issues/151
test!(top_mono, "top", "./fixtures/mono.wasm", "-n", "10");

// // Threaded modules should not cause a panic.
// test!(
// top_threaded_module,
// "top",
// "-n",
// "25",
// "./fixtures/threads.wasm"
// );

// DEV KTM: TEMPORARY WHILE REDUCING BINARY
#[test]
fn top_threaded_module() {
    use std::process::Command;

    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("top")
        .arg("-n")
        .arg("25")
        .arg("./fixtures/threads_test.wasm")
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/all/"))
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(101),);
    // assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);

    let is_error_about_data_count_header = stderr
        .lines()
        .any(|l| l.contains("data count section headers"));
    let contains_correct_panic = stderr.lines().any(|l| l.contains("ir.rs:61:13"));
    assert!(
        contains_correct_panic && is_error_about_data_count_header,
        "incorrect panic!:\n{}",
        stderr
    );
}
