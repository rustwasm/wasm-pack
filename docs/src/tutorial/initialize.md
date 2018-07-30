# Project Initialization

Now that we've installed all of our tools and setup our npm account we can actually start coding!
We'll be writing up a small crate that adds two numbers and outputs the numbers. While this will
be a simple example, we're really trying to focus on how to use wasm-pack. You'll be provided links
to other resources so you can make more complicated code to package and ship them to npm!

Let's get started then! First off run this command to create our project:

```bash
$ cargo new --lib wasm-add
```

This will create a new Rust project in a directory called `wasm-add`. We've also specified that
we're building a library, since we'll be calling this code from JS.

Now just:

```bash
$ cd wasm-add
```

You'll find everything in here ready to get started. First though we'll need to add a dependency to
our code and make a few small changes. Open up your `Cargo.toml` file. You should see something like
this inside:

```toml
[package]
name = "wasm-add"
version = "0.1.0"
authors = ["Michael Gattozzi <mgattozzi@gmail.com>"]

[dependencies]
```

This configuration file sets up everything we need to get started but we'll need a few extra fields
and settings to get this to work for wasm and be ready for npm

```toml
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
wasm-bindgen = "0.2"
```

First off lets look at the last three fields added to the package section `description`, `license`,
and `repository`. npm requires this metadata and so `wasm-pack` won't package your code up until you
have them set. There are more fields that you can add that are more specific to `crates.io` that you
can find [here](https://doc.rust-lang.org/cargo/reference/manifest.html) but for the sake of this
tutorial that's all you need for that section.

You'll also notice we add a new section titled `[lib]`. In here we added this line:

```toml
crate-type = ["cdylib"]
```

Normally Rust compiles the code for the library in a format meant for other Rust packages. We want
our code to work with wasm though! We specify that it's a dynamic library that's C compatible. This
sounds a bit weird but the `wasm32` target will know to interpret this option and instead produce
a wasm binary properly. This is meant to get `cargo` to pass the right parameters to the compiler!

Alright the last thing we added was this to the `[dependencies]` section:

```toml
wasm-bindgen = "0.2"
```

This is the `wasm-bindgen` crate. We'll be using it very shortly to make our functions work nicely
with wasm and not have to worry about a lot of nitty gritty details.

We've got our package's metadata all set up, so let's actually write some code!
