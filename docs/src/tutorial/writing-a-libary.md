# Rust Code

If you open up `src/lib.rs` you should see a file that looks like this:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
```

Let's quickly modify the test suite to work for what we'll be doing. It should look like this:

```rust
#[test]
fn it_works() {
    assert_eq!(add(2, 2), 4);
}
```

We'll use this later to make sure our `add` function works!

Now we need to add this to the top of the file:

```rust
#![feature(use_extern_macros, wasm_import_module, wasm_custom_section)]
extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;
```

Let's step through this line by line. First up is the list of nightly features. We're enabling this for
the whole crate. What this means is that we will later tag code with an attribute and this will
allow Rust to generate code that we don't have to write by hand. In our case it'll use
`wasm-bindgen.` It should be noted that `#![feature(...)]` implies using the nightly
compiler. This gated feature will hopefully be stabilized and landed soon so that you won't need it!

`wasm-bindgen` knows how to make code that works well with wasm so we don't have to
worry about it too much and just write Rust code for the most part. If you want to know the full
extent of its capabilities check out the README on its repo which can be found
[here](https://github.com/alexcrichton/wasm-bindgen). For our purposes we need to know that if we
want functions to work with wasm easily we'll need it.

The next line says we're importing the `wasm-bindgen` crate and the line after that imports the
prelude from `wasm-bindgen`. The `extern crate` call lets the compiler know what crates to link in
and the `prelude` contains all the types and functions that `wasm-bindgen` needs to work properly!

Cool let's import the `alert` function from JS so that we can call it in our Rust code!

```rust
#[wasm_bindgen]
extern {
    fn alert(s: &str);
}
```

Alright so we have our external bit of code and we have everything imported so let's write the
actual `add` function, as well as an `add_alert` function that will use `add` in itself but also
call `alert` to print out the results before returning the value.

```rust
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[wasm_bindgen]
pub fn alert_add(a: i32, b: i32) -> i32 {
    let c = add(a, b);
    alert(&format!("Hello from Rust! {} + {} = {}", a, b, c));
    c
}
```

These functions are fairly straightforward if you're familiar with Rust, but if you're not we'll walk
through it. Both functions take a value `a` and a value `b`. We have said that both are 32 bit
integers (`i32`). We then say both will return an `i32`. The last line in a function returns the value
if there is no semicolon. So in the `add` function the value of `a + b` gets calculated and it's
value is returned! In the case of `alert_add` we store the value of the `add` function we just made
into the variable `c`. We then call `alert` saying what the add operation looked like and what the
value was! We then return what was inside `c`. Neat!

This is all the Rust code we need to write. Your `lib.rs` file should look like this by now:

```rust
#![feature(use_extern_macros, wasm_import_module, wasm_custom_section)]
extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[wasm_bindgen]
pub fn alert_add(a: i32, b: i32) -> i32 {
    let c = add(a, b);
    alert(&format!("Hello from Rust! {} + {} = {}", a, b, c));
    c
}

#[test]
fn it_works() {
    assert_eq!(add(2, 2), 4);
}
```

Just to make sure that `add` works we'll run the test we wrote earlier:

```bash
$ cargo test
```

You should get output that looks sort of like this:

```bash
   Compiling wasm-add v0.1.1 (file:///home/michael/Code/wasm-add)
    Finished dev [unoptimized + debuginfo] target(s) in 0.54 secs
     Running target/debug/deps/wasm_add-5d5676e23e39dbea
running 1 test
test it_works ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Yay it all works! Notice we didn't add a test for `alert_add`. This is because Rust won't know what
`alert` is unless the wasm code is running in the browser! Don't worry though. Once we package this
code up and upload it to npm we'll then test out that function to make sure everything works like we
expect it too!

You can find all of the above code [here](https://github.com/mgattozzi/wasm-add).
