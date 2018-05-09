(module
    ;; -------------------------------------------------------------------------
    ;; This is a WebAssembly text file that can be compiled in a wasm module to
    ;; test the `twiggy garbage` command. This file contains exported functions,
    ;; as well as unreachable functions of different sizes.
    ;; -------------------------------------------------------------------------
    ;; NOTE: The test cases expect that this module is compiled with debug
    ;; names written to the binary file, which affects the size percentages.
    ;; Compile this file using the following command:
    ;;
    ;; wat2wasm --debug-names garbage.wat -o garbage.wasm
    ;; -------------------------------------------------------------------------

    ;; This unused function is called by 'unusedAddOne'. Push 1 onto the stack.
    (func $unusedChild (result i32)
        i32.const 1)

    ;; This unused function will call `unusedChild`, and return `val + 1`.
    (func $unusedAddOne (param $val i32) (result i32)
        get_local $val
        call $unusedChild
        i32.add)

    ;; This unused function adds three numbers, and returns the result.
    (func $unusedAddThreeNumbers
        (param $first i32) (param $second i32) (param $third i32) (result i32)
            get_local $first
            get_local $second
            i32.add
            get_local $third
            i32.add
    )

    ;; This function exists to test that reachable items are not shown.
    (func $add (param $lhs i32) (param $rhs i32) (result i32)
        get_local $lhs
        get_local $rhs
        i32.add
    )

    ;; Export only the `add` function.
    (export "add" (func $add))
)