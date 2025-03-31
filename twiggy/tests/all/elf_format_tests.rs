test!(
    elf_top_25_hello_world_rs,
    "top",
    "-n",
    "25",
    "./fixtures/hello_elf"
);

test!(elf_top_hello_world_rs, "top", "./fixtures/hello_elf");

test!(
    elf_paths,
    "paths",
    "./fixtures/hello_elf",
    "addr2line::render_file::h8b2b27d4ac1b7166",
    "-d",
    "3" //"-f",
        //"json"
);

test!(
    elf_paths2,
    "paths",
    "./fixtures/hello_elf",
    "main",
    "-d",
    "3" //"-f",
        //"json"
);

test!(
    elf_dominators,
    "dominators",
    "./fixtures/hello_elf",
    "-d",
    "3" //"-f",
        //"json"
);

test!(
    elf_dominators2,
    "dominators",
    "./fixtures/hello_elf",
    "main",
    "-d",
    "3" //"-f",
        //"json"
);

test!(
    elf_dominators3,
    "dominators",
    "./fixtures/hello_elf",
    "rust_panic",
    "-d",
    "3" //"-f",
        //"json"
);
