# wasm-pack build

The `wasm-pack build` command creates the files neccessary for JavaScript
interoperability and for publishing a package to npm. This involves compiling
your code to wasm and generating a pkg folder. This pkg folder will contain the
wasm binary, a JS wrapper file, your `README`, and a `package.json` file.

The `pkg` directory is automatically `.gitignore`d by default, since it contains
build artifacts which are not intended to be checked into version
control.<sup>[0](#footnote-0)</sup>

## Path

The `wasm-pack build` command can be given an optional path argument, e.g.:

```
wasm-pack build examples/js-hello-world
```

This path should point to a directory that contains a `Cargo.toml` file. If no
path is given, the `build` command will run in the current directory.

## Profile

The `build` command accepts an optional profile argument: one of `--dev`,
`--profiling`, or `--release`. If none is supplied, then `--release` is used.

Th controls whether debug assertions are enabled, debug info is generated, and
which (if any) optimizations are enabled.

| Profile       | Debug Assertions | Debug Info | Optimizations | Notes                                 |
|---------------|------------------|------------|---------------|---------------------------------------|
| `--dev`       | Yes              | Yes        | No            | Useful for development and debugging. |
| `--profiling` | No               | Yes        | Yes           | Useful when profiling and investigating performance issues. |
| `--release`   | No               | No         | Yes           | Useful for shipping to production.    |

The `--dev` profile will build the output package using cargo's [default
non-release profile][cargo-profile-sections-documentation]. Building this way is
faster but applies few optimizations to the output, and enables debug assertions
and other runtime correctness checks. The `--profiling` and `--release` profiles
use cargo's release profile, but the former enables debug info as well, which
helps when investigating performance issues in a profiler.

The exact meaning of the profile flags may evolve as the platform matures.

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
| `no-modules`  | Outputs JS that use no modules. `browser` key in `package.json`. |
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

## Extra options

The `build` command can pass extra options straight to `cargo build` even if they are not
supported in wasm-pack. To use them you should add standalone `--` argument at the very
end of your command, and all the arguments you want to pass to cargo should go after.
For example to build previous example using unstable cargo offline feature:

```
wasm-pack build examples/js-hello-world --mode no-install -- -Z offline
```

<hr style="font-size: 1.5em; margin-top: 2.5em"/>

<sup id="footnote-0">0</sup> If you need to include additional assets in the pkg
directory and your NPM package, we intend to have a solution for your use case
soon. [↩](#wasm-pack-build)
