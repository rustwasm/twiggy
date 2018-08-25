test!(cpp_monos, "monos", "./fixtures/cpp-monos.wasm");

test!(monos, "monos", "./fixtures/monos.wasm");

test!(
    monos_maxes,
    "monos",
    "./fixtures/monos.wasm",
    "-m",
    "2",
    "-n",
    "1"
);

test!(monos_only_generics, "monos", "./fixtures/monos.wasm", "-g");

test!(
    monos_wasm_csv,
    "monos",
    "./fixtures/monos.wasm",
    "-f",
    "csv"
);

test!(monos_all, "monos", "./fixtures/monos.wasm", "-a");

test!(
    monos_only_all_generics,
    "monos",
    "./fixtures/monos.wasm",
    "-g",
    "-a"
);

test!(
    monos_all_generics,
    "monos",
    "./fixtures/monos.wasm",
    "--all-generics"
);

test!(
    monos_all_monos,
    "monos",
    "./fixtures/monos.wasm",
    "--all-monos"
);

test!(
    monos_json,
    "monos",
    "./fixtures/monos.wasm",
    "-m",
    "2",
    "-n",
    "1",
    "-f",
    "json"
);
