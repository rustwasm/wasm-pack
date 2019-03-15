# Using a Template

You can create a new Rust-WebAssembly project by using the [rustwasm wasm-pack-template].

To so do, you'll need the `cargo-generate` tool. To install `cargo-generate`:

```
cargo install cargo-generate
```

Then run:

```
cargo generate --git https://github.com/rustwasm/wasm-pack-template
```

You will be prompted to give your project a name. Once you do, you will have a directory
with a new project, ready to go. We'll talk about what's been included in this template
further in this guide.

[rustwasm wasm-pack-template]: https://github.com/rustwasm/wasm-pack-template
