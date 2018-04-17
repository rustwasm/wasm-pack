# Contributing

This project is currently not ready for code contributions! However, please do
take a look at the issue tracker. Issues marked "question" are excellent places
to share feedback.

## Submitting a PR

Before you submit your pull request, check that you have completed all of the
steps mentioned in the pull request template. Link the issue that your pull
request is responding to, and format your code using [rustfmt][rustfmt].

### Configuring rustfmt

Before submitting code in a PR, make sure that you have formatted the codebase
using [rustfmt][rustfmt]. `rustfmt` is a tool for formatting Rust code, which
helps keep style consistent across the project. If you have not used `rustfmt`
before, it is not too difficult.

If you have not already configured `rustfmt` for the
nightly toolchain, it can be done using the following steps:

**1. Use Nightly Toolchain**

Use the `rustup override` command to make sure that you are using the nightly
toolchain. Run this command in the `wasm-pack` directory you cloned.

```sh
rustup override set nightly
```

**2. Add the rustfmt component**

Install the most recent version of `rustfmt` using this command:

```sh
rustup component add rustfmt-preview --toolchain nightly
```

**3. Running rustfmt**

To run `rustfmt`, use this command:

```sh
cargo +nightly fmt
```

[rustfmt]: https://github.com/rust-lang-nursery/rustfmt

## Conduct

As mentioned in the readme file, this project is a part of the
[rust-wasm][rust-wasm] group. As such, contributors should be sure to follow
the rust-wasm group's [code of conduct][rust-wasm-coc], as well as the Rust
language [code of conduct][rust-coc].

[rust-wasm]: https://github.com/rust-lang-nursery/rust-wasm
[rust-wasm-coc]:working://github.com/rust-lang-nursery/rust-wasm/blob/master/CODE_OF_CONDUCT.md
[rust-coc]: https://www.rust-lang.org/en-US/conduct.html

