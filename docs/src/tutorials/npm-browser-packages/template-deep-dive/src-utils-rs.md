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
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}
```

As described in the preceding section, this invocation of the `cfg_if!`
tests whether the `console_error_panic_hook` feature is enabled at compile time,
replacing with the statements in the `if` block or with those in the `else`
block. These two cases are:

```rust
pub use self::console_error_panic_hook::set_once as set_panic_hook;
```

This `use` statement means the function
`self::console_error_panic_hook::set_once` can now be accessed more conveniently
as `set_panic_hook`. With `pub`, this function will be accessible
outside of the `utils` module as `utils::set_panic_hook`.

```rust
#[inline]
pub fn set_panic_hook() {}
```

Here, `set_panic_hook` is defined to be an empty inline function. The inline
annotation here means that whenever the function is called the function call is
replaced with the body of the function, which is for `set_panic_hook` nothing!
This allows the use of `set_panic_hook` without any run-time or code-size
performance penalty if the feature is not enabled.

## 2. What is `console_error_panic_hook`?

The [crate `console_error_panic_hook`][ceph] allows debugging Rust panic
messages in a web browser, making it much easier to debug WebAssembly code.

Let's compare what happens when Rust code panics before and after enabling the
feature:

**Before:** `"RuntimeError: Unreachable executed"`

**After:** `"panicked at 'index out of bounds: the len is 3 but the index is 4', libcore/slice/mod.rs:2046:10"`

To do this, a [panic hook] is configured that logs panics to the
developer console via the JavaScript `console.error` function.

Note though that `console_error_panic_hook` is not entirely automatic, so you'll
need to make sure that `utils::set_panic_hook()` is called before any of our
code runs (and it's safe to run `set_panic_hook` many times).

For more details, see the [`console_error_panic_hook`
repository](https://github.com/rustwasm/console_error_panic_hook).

[ceph]: https://crates.io/crates/console_error_panic_hook
[panic hook]: https://doc.rust-lang.org/std/panic/fn.set_hook.html
