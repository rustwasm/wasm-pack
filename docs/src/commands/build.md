# wasm-pack build

The `wasm-pack build` command creates the files neccessary for JavaScript
interoperability and for publishing a package to npm. This involves compiling
your code to wasm and generating a pkg folder. This pkg folder will contain the
wasm binary, a JS wrapper file, your `README`, and a `package.json` file.

## Path

The `wasm-pack build` command can be given an optional path argument, e.g.:

```
wasm-pack build examples/js-hello-world
```

This path should point to a directory that contains a `Cargo.toml` file. If no
path is given, the `build` command will run in the current directory.

## Debug

The init command accepts an optional `--debug` argument. This will build the
output package using cargo's
[default non-release profile][cargo-profile-sections-documentation]. Building
this way is faster but applies few optimizations to the output, and enables
debug assertions and other runtime correctness checks.

The exact meaning of this flag may evolve as the platform matures.

[cargo-profile-sections-documentation]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-profile-sections

## Target

The `build` command accepts a `--target` argument. This will customize the output files
to align with a particular type of JS module. This allows wasm-pack to generate either
ES6 modules or CommonJS modules for use in browser and in NodeJS. Defaults to `browser`.
The options are:

```
wasm-pack build --target nodejs
```

| Option    | Description                                                                                                     |
|-----------|-----------------------------------------------------------------------------------------------------------------|
| `nodejs`  | Outputs JS that uses CommonJS modules, for use with a `require` statement. `main` key in `package.json`. |
| `nomodules`  | Outputs JS that use no modules. `browser` key in `package.json`. |
| `browser` | Outputs JS that uses ES6 modules, primarily for use with `import` statements and/or bundlers such as `webpack`. `module` key in `package.json`. `sideEffects: false` by default. |

## Scope

The init command also accepts an optional `--scope` argument. This will scope
your package name, which is useful if your package name might conflict with
something in the public registry. For example:

```
wasm-pack build examples/js-hello-world --scope test
```

This command would create a `package.json` file for a package called
`@test/js-hello-world`. For more information about scoping, you can refer to
the npm documentation [here][npm-scope-documentation].

[npm-scope-documentation]: https://docs.npmjs.com/misc/scope

## Mode

The `build` command accepts an optional `--mode` argument.
```
wasm-pack build examples/js-hello-world --mode no-install
```

| Option        | Description                                                                              |
|---------------|------------------------------------------------------------------------------------------|
| `no-install`  | `wasm-pack init` implicitly and create wasm binding  without installing `wasm-bindgen`.  |
| `normal`      | do all the stuffs of `no-install` with installed `wasm-bindgen`.                         |
