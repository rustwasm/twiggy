test!(
    elf_top_25_hello_world_rs,
    "top",
    "-n",
    "25",
    "./fixtures/hello_elf"
);

test!(elf_top_hello_world_rs, "top", "./fixtures/hello_elf");
