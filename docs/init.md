# wasm-pack init

The `wasm-pack init` command creates the files neccessary for JavaScript
interoperability and for publishing a package to npm. This involves compiling
your code to wasm and generating a pkg folder. This pkg folder will contain the
wasm binary, a JS wrapper file, your `README`, and a `package.json` file.

## Path

The `wasm-pack init` command can be given an optional path argument, e.g.:

```
wasm-pack init examples/js-hello-world
```

This path should point to a directory that contains a `Cargo.toml` file. If no
path is given, the `init` command will run in the current directory.

## Scope

The init command also accepts an optional `--scope` argument. This will scope
your package name, which is useful if your package name might conflict with
something in the public registry. For example:

```
wasm-pack init examples/js-hello-world --scope test
```

This command would create a `package.json` file for a package called
`@test/js-hello-world`. For more information about scoping, you can refer to
the npm documentation [here][npm-scope-documentation].

[npm-scope-documentation]: https://docs.npmjs.com/misc/scope
