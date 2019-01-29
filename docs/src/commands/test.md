# wasm-pack test

The `wasm-pack test` command wraps the [wasm-bindgen-test-runner](https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/index.html)
CLI allowing you to run wasm tests in different browsers without needing to install the different
webdrivers yourself.

```
wasm-pack test --help
```

## Path

The `wasm-pack test` command can be given an optional path argument.

This path should point to a directory that contains a `Cargo.toml` file. If no
path is given, the `test` command will run in the current directory.

```
# Run tests for the current directory's crate
wasm-pack test

# Run tests for a specified crate
wasm-pack test crates/crate-in-my-workspace
```

## Profile

The `test` command accepts an optional profile argument: `--release`.

If none is supplied, then a debug test build will be used.

## Test environment

Choose where to run your tests by passing in any combination of testing environment flags.

`--headless` is useful for running browser tests in a headless browser as part of a CI process.

```
wasm-pack test --node --firefox --chrome --safari --headless
```

## Extra options

The `test` command can pass extra options straight to `cargo test` even if they are not
supported in wasm-pack.

To use them you should add standalone `--` argument at the very
end of your command, and all the arguments you want to pass to cargo should go after.

Here's an example of running only tests that contain the world "apple".

```
wasm-pack test --firefox --headless -- --manifest-path=crates/my-workspace-crate/Cargo.toml apple
```

`cargo test -h` for a list of all options that you can pass through.
