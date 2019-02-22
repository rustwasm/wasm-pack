# Cargo.toml

`Cargo.toml` is the manifest file for Rust's package manager, `cargo`. This file contains
metadata such as name, version, and dependencies for packages, which are call "crates" in Rust.

There's a bunch of metadata that the template gives us, but there are three key parts to discuss:

1. [`crate-type`](#a1-crate-type)
2. [`wasm-bindgen` dependency](#a2-wasm-bindgen-dependency)
3. [`[features]` and `wee_alloc`, `console_error_panic_hook` dependencies](#a3-features-and-wee_alloc-console_error_panic_hook-dependencies)

<hr/>

## 1. `crate-type`

```toml
[lib]
crate-type = ["cdylib", "rlib"]
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

`#[crate_type = "rlib"]` signifies that an intermediate "Rust library" file will be produced. 
This allows tests to use the main crate.

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

## 3. `[features]` and `wee_alloc`, `console_error_panic_hook` dependencies

As part of our effort to design a template that helps people discover useful crates
for their particular use case, this template includes two dependencies that can be
very useful for folks developing Rust-`wasm` crates: `console-error-panic-hook` and
`wee-alloc`.

Because these dependencies are useful primarily in a specific portion of the Rust-`wasm`
crate development workflow, we've also set up a bit of glue code that allows us to include
them both as dependencies, but also allows them to be optionally included.

```toml
[features]
default = ["console_error_panic_hook"]

[dependencies]
cfg-if = "0.1.2"
wasm-bindgen = "0.2"

# The `console_error_panic_hook` crate provides better debugging of panics by
# logging them with `console.error`. This is great for development, but requires
# all the `std::fmt` and `std::panicking` infrastructure, so isn't great for
# code size when deploying.
console_error_panic_hook = { version = "0.1.1", optional = true }

# `wee_alloc` is a tiny allocator for wasm that is only ~1K in code size
# compared to the default allocator's ~10K. It is slower than the default
# allocator, however.
#
# Unfortunately, `wee_alloc` requires nightly Rust when targeting wasm for now.
wee_alloc = { version = "0.4.2", optional = true }
```

[`cfg-if`] allows us to check if certain features are enabled on a Rust crate. We'll
use this crate later to optionally enable `console_error_panic_hook` or
`wee_alloc`.

By default, only `console_error_panic_hook` is enabled. To disable either
feature, we can remove its name from the `default` vector.

To learn more about these features, we discuss them in-depth in the `src/lib.rs` and 
`src/utils.rs` sections.

Briefly, they include:

+ **console_error_panic_hook** for logging panic messages to the developer console.
+ **wee_alloc**, an allocator optimized for small code size.
