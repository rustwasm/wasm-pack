# Rust

`wasm-pack` is a tool written in Rust, and distributed with `cargo`. As a result,
you'll need Rust and `cargo` to use `wasm-pack`.

To install Rust, visit this [page](https://www.rust-lang.org/en-US/install.html).

You can be sure you have Rust and Cargo installed by running:

```
rustc --version
cargo --version
```

### `nightly` Rust

`wasm-pack` depends on `wasm-bindgen` which currently requires Rust features that
have not yet been stabilized. As a result, you'll need to use a nightly version of
Rust to run `wasm-pack`.

You can install the nightly channel by running:

```
rustup install nightly
```

You can configure rustup to always use `nightly` in a directory by running:

```
rustup override set nightly
```
