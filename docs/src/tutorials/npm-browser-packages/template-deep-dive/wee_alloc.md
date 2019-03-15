# wee_alloc

1. [What is `wee_alloc`?](#what-is-wee_alloc)
2. [Enabling `wee_alloc`](#enabling-wee_alloc)
3. [Rust nightly](#rust-nightly)

## What is `wee_alloc`?

Reducing the size of compiled WebAssembly code is important, since it is often transmitted over the Internet or placed on embedded devices.

> `wee_alloc` is a tiny allocator designed for WebAssembly that has a (pre-compression) code-size footprint of only a single kilobyte.

[An analysis](http://fitzgeraldnick.com/2018/02/09/wee-alloc.html) suggests that over half of the bare minimum WebAssembly memory footprint is required by Rust's default memory allocator. Yet, WebAssembly code often does not require a sophisticated allocator, since it often just requests a couple of large initial allocations.

`wee_alloc` trades off size for speed. Although it has a tiny code-size footprint, it is relatively slow if additional allocations are needed.

For even more details, see the [`wee_alloc` repository](https://github.com/rustwasm/wee_alloc).

## Enabling `wee_alloc`

In `lib.rs`, we have the configuration for `wee_alloc` inside a `cfg_if!` macro:

```rust
cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}
```

This code block is intended to initialize `wee_alloc` as the global memory allocator, but only if the `wee_alloc` feature is enabled in `Cargo.toml`.

To do so we need to append `"wee_alloc"` to the `default` vector in `Cargo.toml`. Then, the `cfg_if!` block is replaced with the contents of the `if` block, shown above.

```toml
[features]
default = ["console_error_panic_hook", "wee_alloc"]
```

## Rust nightly

`wee_alloc` currently relies on features only available in Rust nightly. As such it requires you to use the nightly toolchain for compilation. If you have [Rustup](https://rustup.rs/) set up, you can install the nightly toolchain as follows:

```
rustup toolchain add nightly
```

To use `wasm-pack` with Rust nightly run:

```
rustup run nightly wasm-pack build
```
