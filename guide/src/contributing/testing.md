# Testing

```
$ cargo test --all --exclude twiggy-wasm-api
```

## Authoring New Tests

Integration tests live in the `twiggy/tests` directory:

```
twiggy/tests
├── expectations
├── fixtures
└── tests.rs
```

* The `twiggy/tests/tests.rs` file contains the `#[test]` definitions.

* The `twiggy/tests/fixtures` directory contains input binaries for tests.

* The `twiggy/tests/expectations` directory contains the expected output of test
  commands.

## Updating Test Expectations

To automatically update all test expectations, you can run the tests with the
`TWIGGY_UPDATE_TEST_EXPECTATIONS=1` environment variable set. Make sure that you
look at the changes before committing them, and that they match your intentions!
