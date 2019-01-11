# src/utils.rs

The purpose of `utils.rs` is to define the `utils` module, which contains a single function `set_panic_hook`. This function becomes part of the `utils` module in `lib.rs`, as described in the preceding section.

If the `console_error_panic_hook` feature is not enabled, then `set_panic_hook` is defined to be an inlined empty function. So, there is no run-time performance or code-size penalty incurred by its use.

We will discuss:
1. [Defining `set_panic_hook`](#a1-defining-set_panic_hook)
2. [What is `console_error_panic_hook`?](#a2-what-is-console_error_panic_hook)


---

## 1. Defining `set_panic_hook`

```rust
use cfg_if::cfg_if;
```

This allows us to write `cfg_if!` instead of `cfg_if::cfg_if!`, identically to the line in `src/lib.rs`.

```rust
cfg_if! {
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}
```

As described in the preceding section, the macro `cfg_if!` evaluates the `if` statement during compile time. This is possible because it is essentially testing whether `"console_error_panic_hook"` is defined in the `[features]` section of `Cargo.toml`, which is available during compile time.

The entire macro block will either be replaced with the statements in the `if` block or with those in the `else` block. These two cases are now described in turn:

```rust
extern crate console_error_panic_hook;
pub use self::console_error_panic_hook::set_once as set_panic_hook;
```

Due to the `use` statement, the function `self::console_error_panic_hook::set_once` can now be accessed more conveniently as `set_panic_hook`. Due to `pub`, this function will be publicly accessible outside of the `utils` module as `utils::set_panic_hook`.

```rust
#[inline]
pub fn set_panic_hook() {}
```

An inline function replaces the function call with the contents of the function during compile time. Here, `set_panic_hook` is defined to be an empty inline function. This allows the use of `set_panic_hook` without any run-time or code-size performance penalty if the feature is not enabled.

## 2. What is `console_error_panic_hook`?

The crate `console_error_panic_hook` enhances error messages in the web browser. This allows you to easily debug WebAssembly code.

Let's compare error messages before and after enabling the feature:

**Before:** `"RuntimeError: Unreachable executed"`

**After:** `"panicked at 'index out of bounds: the len is 3 but the index is 4', libcore/slice/mod.rs:2046:10"`

To do this, a panic hook for WebAssembly is provided that logs panics to the developer console via the JavaScript `console.error` function. 

Note that although the template sets up the function, your error messages will not automatically be enhanced. To enable the enhanced errors, call the function `utils::set_panic_hook()` in `lib.rs` when your code first runs. The function may be called multiple times if needed.

For more details, see the [`console_error_panic_hook` repository](https://github.com/rustwasm/console_error_panic_hook).
