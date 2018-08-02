# Package Code for npm

We have written our code, so now we need to package it all up. In your project directory run the following
command:

```bash
$ wasm-pack build --scope MYSCOPE
```

where `MYSCOPE` is your npm username. Normally you could just type `wasm-pack build`, but since
other people are doing this tutorial as well, we don't want conflicts with the `wasm-add` package
name! This command when run does a few things:

1. It will compile your code to wasm if you have not already.
2. It will generate an output folder containing the wasm file, a JS wrapper file around the wasm, your README,
   and a `package.json` file. This folder is named `pkg` by default, but can be specified using the `--out-dir` option.

This is everything you need to upload your code to npm! Let's do just that!

First off, you will need to login to npm with the account you made earlier:

```bash
$ wasm-pack login
```

Next you will need to go into the output directory and actually upload the package:

```bash
$ cd pkg
$ npm publish --access=public
```

Normally if things are not scoped you can just do `npm publish`, but if you give it a scope
you will need to tell npm that this is actually public so that it can be published. We need to do that here
because we gave our packages a scope to avoid conflicting with each other! Next up is actually running
the code and verifying we got it from npm and how we can use that code.
