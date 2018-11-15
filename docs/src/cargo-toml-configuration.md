# `Cargo.toml` Configuration

`wasm-pack` can be configured via the `package.metadata.wasm-pack` key in
`Cargo.toml`. Every option has a default, and is not required.

There are three profiles: `dev`, `profiling`, and `release`. These correspond to
the `--dev`, `--profiling`, and `--release` flags passed to `wasm-pack build`.

The available configuration options and their default values are shown below:

```toml
[package.metadata.wasm-pack.profile.dev.wasm-bindgen]
# Should we enable wasm-bindgen's debug assertions in its generated JS glue?
debug-js-glue = true
# Should wasm-bindgen demangle the symbols in the "name" custom section?
demangle-name-section = true
# Should we emit the DWARF debug info custom sections?
dwarf-debug-info = false

[package.metadata.wasm-pack.profile.profiling.wasm-bindgen]
debug-js-glue = false
demangle-name-section = true
dwarf-debug-info = false

[package.metadata.wasm-pack.profile.release.wasm-bindgen]
debug-js-glue = false
demangle-name-section = true
dwarf-debug-info = false
```
