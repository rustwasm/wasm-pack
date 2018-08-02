# Rust

`wasm-pack` is a Command Line Interface tool written in Rust, and distributed with `cargo`.
As a result, you'll need Rust and `cargo` to use `wasm-pack`.

### Installing Rust and Cargo

To install Rust, visit this [page](https://www.rust-lang.org/en-US/install.html), which will
walk you through installing Rust and `cargo` on your machine using a tool called `rustup`.

To confirm you have Rust and `cargo` installed, run:

```
rustc --version
cargo --version
```

### Rust Versions

`wasm-pack` depends on a library called `wasm-bindgen`. `wasm-bindgen` requires that you use
Rust 1.30.0 or higher. This version is currently only available on the `nightly` or `beta`
channels.

To get the correct version of Rust, you'll use `rustup` a Rust version manager that comes
bundled with Rust. Run this command to install the latest Rust on the `beta` channel:

```
rustup install beta
```

You can set your project directory to always use this version of Rust by running:

```
rustup override set beta
```
