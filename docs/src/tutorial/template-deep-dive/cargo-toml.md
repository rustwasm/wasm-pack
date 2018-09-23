# Cargo.toml

`Cargo.toml` is the manifest file for Rust's package manager, `cargo`. This file contains
metadata such as name, version, and dependencies for packages, which are call "crates" in Rust.

There's a bunch of metadata that the template gives us, but there are 3 key parts to discuss:

- [`crate-type`](#a1-crate-type)
- [`wasm-bindgen` dependency](#a2-wasm-bindgen-dependency)
- [`[features]` and `wee-alloc`, `console-error-panic-hook` dependencies](#a3-features-and-wee-alloc-console-error-panic-hook-dependencies)

<hr/>

## 1. `crate-type`

```toml
[lib]
crate-type = ["cdylib"]
```

A Rust-`wasm` crate is a bit different from a normal crate, and as a result, we need to note
this in our `Cargo.toml`.

When `cargo` is told to build a project, or compilation is otherwise done on a Rust project,
the Rust compiler will need to link crates together, using a particular method, either
staticly or dynamically. The two types of crate that you are likely most familiar with are
`#[crate_type = "bin"]` and `#[crate_type = "lib"]`, which are the crate types that largely
represent the difference between Rust application projects and Rust libraries.

`#[crate_type = "cdylib"]` signifies that you'd like the compiler to create a dynamic system
library. This type of library is suited for situations where you'd like to compile Rust code
as a dynamic library to be loaded from another language. In our case, we'll be compiling to a
`.wasm` file, but this output type will create `*.so` files on Linux, `*.dylib` files on
macOS, and `*.dll` files on Windows in non-`wasm` circumstances.

You can read more about linking and crate types, [here](https://doc.rust-lang.org/reference/linkage.html).

## 2. `wasm-bindgen` dependency

`wasm-bindgen` is our most important dependency. This package allows us to use the
`#[wasm-bindgen]` attribute to tag code that represents the interface we want between
our JavaScript and Rust-generated `wasm`. We can import JS and export Rust by using this
attribute.

```toml
wasm-bindgen = "0.2"
```

We'll see more about how to use this library when we discuss what has been generated in `lib.rs`.

⚠️ If you are coming from JavaScript, you might note that when we add the dependency
there is no `^` or `~` symbol- it looks like we're locking to the `0.2` version. 
However, that's not the case! In Rust, the `^` is implied.

## 3. `[features]` and `wee-alloc`, `console-error-panic-hook` dependencies

As part of our effort to design a template that helps people discover useful crates
for their particular use case, this template includes 2 dependencies that can be
very useful for folks developing Rust-`wasm` crates: `console-error-panic-hook` and
`wee-alloc`.

Because these dependencies are useful primarily in a specifc portion of the Rust-`wasm`
crate development workflow, we've also set up a bit of glue code that allows us to include
them both as dependences, but allowing for them to be optionally included.

```toml
[features]
default-features = ["console_error_panic_hook", "wee_alloc"]

[dependencies]
cfg-if = "0.1.2"

# The `console_error_panic_hook` crate provides better debugging of panics by
# logging them with `console.error`. This is great for development, but requires
# all the `std::fmt` and `std::panicking` infrastructure, so isn't great for
# code size when deploying.
console_error_panic_hook = { version = "0.1.1", optional = true }

# `wee_alloc` is a tiny allocator for wasm that is only ~1K in code size
# compared to the default allocator's ~10K. It is slower than the default
# allocator, however.
wee_alloc = { version = "0.4.1", optional = true }
```

[`cfg-if`] allows us to check if certain features are enabled on a rust crate. We'll
use this crate in `utils.rs` to optionally enable `console_error_panic_hook` or
`wee_alloc`. By default, we have them enabled. To disable them, we can remove their
entry from the `default-features` vector.

To learn more about these features, we discuss them in depth in the `utils.rs` section.
