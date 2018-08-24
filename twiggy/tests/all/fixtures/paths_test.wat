(module
    ;; ------------------------------------------------------------------------
    ;; This is a WebAssembly text file that can be compiled in a wasm module to
    ;; test the `twiggy paths` command. This intends to provide a non-trivial
    ;; structure of call paths for testing purposes.
    ;;
    ;; The call path is shown in the ascii diagram below with exported
    ;; functions enclosed in braces, and unexported functions in quotes.
    ;;
    ;;                [awoo]
    ;;                  |
    ;;                  v
    ;;     [woof]     [bark]
    ;;       |         | |
    ;;       |  -------- |
    ;;       |  |        |
    ;;       v  v        v
    ;; 'calledOnce' 'calledTwice'
    ;; ------------------------------------------------------------------------
    ;; NOTE: The test cases expect that this module is compiled with debug
    ;; names written to the binary file, which affects the size percentages.
    ;; Compile this file using the following command:
    ;;
    ;; wat2wasm --debug-names paths_test.wat -o paths_test.wasm
    ;; -------------------------------------------------------------------------


    ;; This function is called once, by 'woof'.
    (func $calledOnce (result i32)
        i32.const 1)

    ;; This function is called twice, by 'bark' and 'woof'.
    (func $calledTwice (result i32)
        i32.const 2)

    (func $bark (result i32)
        call $calledTwice)

    (func $woof (result i32)
        call $calledOnce
        call $calledTwice
        i32.add)

    (func $awoo (result i32)
        call $bark)

    (export "awoo" (func $awoo))
    (export "bark" (func $bark))
    (export "woof" (func $woof))
)
