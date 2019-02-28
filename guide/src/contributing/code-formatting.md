# Code Formatting

We use [`rustfmt`](https://github.com/rust-lang-nursery/rustfmt) to enforce a
consistent code style across the whole code base.

You can install the latest version of `rustfmt` with this command:

```
$ rustup update
$ rustup component add rustfmt --toolchain stable
```

Ensure that `~/.rustup/toolchains/$YOUR_HOST_TARGET/bin/` is on your `$PATH`.

Once that is taken care of, you can (re)format all code by running this command
from the root of the repository:

```
$ cargo fmt --all
```

# Linting

We use [`clippy`](https://github.com/rust-lang/rust-clippy) to lint the codebase.
This helps avoid common mistakes, and ensures that code is correct,
performant, and idiomatic.

You can install the latest version of `clippy` with this command:

```
$ rustup update
$ rustup component add clippy --toolchain stable
```

Once that is complete, you can lint your code to check for mistakes by running
this command from the root of the repository:

```
$ cargo clippy
```
