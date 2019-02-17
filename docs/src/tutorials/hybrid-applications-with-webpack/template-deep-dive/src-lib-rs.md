# src/lib.rs

`lib.rs` is the template's main source file. In the
rust-webpack template, the `lib.rs` is generated inside the
`crate` directory. Libraries in Rust are commonly called
crates and for this template, the Rust code written in this
file will be compiled as a library.

Our project contains four key parts:

- [`#[wasm_bindgen] functions`](#a1-wasm_bindgen-functions)
- [Crate imports](#a2-crate-imports)
- [`wee_alloc` optional dependency](#a3-wee_alloc-optional-dependency)
        - [What is `wee_alloc`?](#what-is-wee_alloc)
- [Defining `set_panic_hook`](#a4-defining-set_panic_hook)
- [`web-sys` features](#a5-web-sys-features)

---

We'll start with the most important part of this `lib.rs`
file -- the `#[wasm_bindgen]` functions. In the rust-webpack
template, `lib.rs` will be the only place you need to modify
and add Rust code.

## 1. `#[wasm_bindgen]` functions

The `#[wasm_bindgen]` attribute indicates that the function
below it will be accessible both in JavaScript and Rust.

```rust
// Called by our JS entry point to run the example.
#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    // ...
    Ok(())
}
```

If we were to write the `run` function without the
`#[wasm_bindgen]` attribute, then `run` would not be easily
accessible within JavaScript. Furthermore, we wouldn't be
able to natively convert certain types such as `()` between
JavaScript and Rust. So, the `#[wasm_bindgen]` attribute
allows `run` to be called from JavaScript.

This is all you need to know to interface with JavaScript!
If you are curious about the rest, read on.

## 2. Crate imports

```rust
#[macro_use]
extern crate cfg_if;
extern crate web_sys;
extern crate wasm_bindgen;
```

In `Cargo.toml`, we included the crates `cfg_if`, `web_sys`,
and `wasm_bindgen` as project dependencies.

Here, we explicitly declare that these crates will be used
in `lib.rs`.

```rust
use wasm_bindgen::prelude::*;
```

`use` allows us to conveniently refer to parts of a crate or
module. For example, suppose the crate `wasm_bindgen`
contains a function `func`. It is always possible to call
this function directly by writing `wasm_bindgen::func()`.
However, this is often tedious to write. If we first specify
`use wasm_bindgen::func;`, then `func` can be called by just
writing `func()` instead.

In our `use` statement above we further specify a `prelude`
module. Many modules contain a "prelude", a list of things
that should be automatically imported. This allows common
features of the module to be conveniently accessed without a
lengthy prefix. For example, in this file we can use
`#[wasm_bindgen]` only because it is brought into scope by
the prelude.

The asterisk at the end of this `use` indicates that
everything inside the module `wasm_bindgen::prelude` (i.e.
the module `prelude` inside the crate `wasm_bindgen`) can be
referred to without prefixing it with
`wasm_bindgen::prelude`.

For example, `#[wasm_bindgen]` could also be written as
`#[wasm_bindgen::prelude::wasm_bindgen]`, although this is
not recommended.

One other point of interest is how we import the `cfg_if!`
macro.

```rust
#[macro_use]
extern crate cfg_if;
```

The `#[macro_use]` attribute imports the `cfg_if!` macro the
same way a `use` statement imports functions.


## 3. `wee_alloc` optional dependency

```rust
cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}
```

This code block is intended to initialize `wee_alloc` as the
global memory allocator, but only if the `wee_alloc` feature
is enabled in `Cargo.toml`.

We immediately notice that `cfg_if!` is a macro because it
ends in `!`, similarly to other Rust macros such as
`println!` and `vec!`. A macro is directly replaced by other
code during compile time.

During compile time, `cfg_if!` evaluates the `if` statement.
This tests whether the feature `wee_alloc` is present in the
`[features]` section of `Cargo.toml` (among other possible
ways to set it).

As we saw earlier, the `default` vector in `[features]` only
contains `"console_error_panic_hook"` and not `"wee_alloc"`.
So, in this case, the `cfg_if!` block will be replaced by no
code at all, and hence the default memory allocator will be
used instead of `wee_alloc`.

```rust
extern crate wee_alloc;
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
```

However, suppose `"wee_alloc"` is appended to the `default`
vector in `Cargo.toml`. Then, the `cfg_if!` block is instead
replaced with the contents of the `if` block, shown above.

This code sets the `wee_alloc` allocator to be used as the
global memory allocator.

### What is `wee_alloc`?

Reducing the size of compiled WebAssembly code is important,
since it is often transmitted over the Internet or placed on
embedded devices.

> `wee_alloc` is a tiny allocator designed for WebAssembly
> that has a (pre-compression) code-size footprint of only a
> single kilobyte.

[An analysis](http://fitzgeraldnick.com/2018/02/09/wee-alloc.html)
suggests that over half of the bare minimum WebAssembly
memory footprint is required by Rust's default memory
allocator. Yet, WebAssembly code often does not require a
sophisticated allocator, since it often just requests a
couple of large initial allocations.

`wee_alloc` trades off size for speed. Although it has a
tiny code-size footprint, it is relatively slow if
additional allocations are needed.

For more details, see the
[`wee_alloc` repository](https://github.com/rustwasm/wee_alloc).

## 4. Defining `set_panic_hook`

```rust
cfg_if! {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        use console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        fn set_panic_hook() {}
    }
}
```

As described in the `wee_alloc` section, the macro `cfg_if!`
evaluates the `if` statement during compile time. This is
possible because it is essentially testing whether
`"console_error_panic_hook"` is defined in the `[features]`
section of `Cargo.toml`, which is available during compile
time.

The entire macro block will either be replaced with the
statements in the `if` block or with those in the `else`
block. These two cases are now described in turn:

```rust
extern crate console_error_panic_hook;
use console_error_panic_hook::set_once as set_panic_hook;
```

Due to the `use` statement, the function
`console_error_panic_hook::set_once` can now be
accessed more conveniently as `set_panic_hook`.

```rust
#[inline]
fn set_panic_hook() {}
```

An inline function replaces the function call with the
contents of the function during compile time. Here,
`set_panic_hook` is defined to be an empty inline function.
This allows the use of `set_panic_hook` without any run-time
or code-size performance penalty if the feature is not
enabled.

### What is `console_error_panic_hook`?

The crate `console_error_panic_hook` enhances error messages
in the web browser. This allows you to easily debug
WebAssembly code.

Let's compare error messages before and after enabling the
feature:

**Before:** `"RuntimeError: Unreachable executed"`

**After:** `"panicked at 'index out of bounds: the len is 3
but the index is 4', libcore/slice/mod.rs:2046:10"`

To do this, a panic hook for WebAssembly is provided that
logs panics to the developer console via the JavaScript
`console.error` function.

Note that although the template sets up the function, your
error messages will not automatically be enhanced. To enable
the enhanced errors, call the function `set_panic_hook()` in
`lib.rs` when your code first runs. The function may be
called multiple times if needed.

For more details, see the
[`console_error_panic_hook` repository](https://github.com/rustwasm/console_error_panic_hook).

## 5. `web-sys` features

The `web-sys` crate enables us to access elements in web
browsers.

By looking at the generated code, we can see we're using a
few of the elements provided by the API in the `web-sys`
crate.

```rust
pub fn run() -> Result<(), JsValue> {
    set_panic_hook();

    let window = web_sys::window().expect("should have a Window");
    let document = window.document().expect("should have a Document");

    let p: web_sys::Node = document.create_element("p")?.into();
    p.set_text_content(Some("Hello from Rust, WebAssembly, and Webpack!"));

    let body = document.body().expect("should have a body");
    let body: &web_sys::Node = body.as_ref();
    body.append_child(&p)?;

    Ok(())
}
```

Here we're accessing the window of the web browser and
elements inside the document that allow us to put text
inside the web browser when we run this example.

When this code is run and we look at the output in our web
browser, we are greeted with text in a `p` element in the
body of the document that says ``"Hello from Rust,
WebAssembly, and Webpack!"``

In the `Cargo.toml`, we enable the specific features we want
to use by listing them in the features array. Our generated
`Cargo.toml` from the rust-webpack template gives us:
```toml
[dependencies.web-sys]
version = "0.3"
features = [
  "Document",
  "Element",
  "HtmlElement",
  "Node",
  "Window",
]
```

You can include more features to access more bindings to the
web that the browser provides.  You can learn more about
what the `web-sys` crate has to offer
[here](https://rustwasm.github.io/wasm-bindgen/api/web_sys/).
