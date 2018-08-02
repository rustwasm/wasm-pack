# Manual Setup

You can create a new Rust project named `my-lib` using this command.

```
cargo new --lib my-lib
```

The `--lib` flag specifies that the project is a library, which is important
because we will be calling this code from JavaScript.

#### Cargo.toml changes

You will need to add `wasm-bindgen` to your `Cargo.toml` in the dependencies
section. `wasm-bindgen` is a tool that facilitates interoperability between
wasm modules and JavaScript.

Next, add a `[lib]` section, with a new field named `crate-type` set to
`"cdylib"`. This specifies that the library is a C compatible dynamic library,
which helps `cargo` pass the correct flags to the Rust compiler when targeting
`wasm32`.

After making these changes, your `Cargo.toml` file should look something like
this:

```
[package]
name = "wasm-add"
version = "0.1.0"
authors = ["Michael Gattozzi <mgattozzi@gmail.com>"]
description = "Code used to demonstrate how to use wasm-pack"
license = "MIT/Apache-2.0"
repository = "https://github.com/mgattozzi/wasm-add"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen="0.2"
```
