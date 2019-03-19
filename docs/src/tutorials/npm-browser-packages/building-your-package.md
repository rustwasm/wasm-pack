# Building your package

We've written our code so now we need to package it all up.

We are writing a package that should be used in the browser, so we run this in our terminal:

```bash
$ wasm-pack build --scope MYSCOPE
```

If you were writing a package that should be used in Node.js (with CommonJS modules, e.g. `require`),
you would run this in your terminal:

```bash
$ wasm-pack build --scope MYSCOPE --target nodejs
```

where `MYSCOPE` is your npm username. Normally you could just type `wasm-pack init` but since
other people are doing this tutorial as well we don't want conflicts with the `wasm-add` package
name! This command when run does a few things:

1. It'll compile your code to wasm if you haven't already
2. It'll generate a pkg folder with the wasm file, a JS wrapper file around the wasm, your README,
   and a `package.json` file.
