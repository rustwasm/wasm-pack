# Building your project

We are writing a package that should be used in the browser, so we run this in our terminal:

```bash
$ wasm-pack build
```

If you were writing a package that should be used in Node.js (with CommonJS modules, e.g. `require`),
you would run this in your terminal:

```bash
$ wasm-pack build --target nodejs
```

This command does a few things when run:

1. It'll compile your code to wasm if you haven't already
2. It'll generate a `pkg` folder. Inside there will be:
    - a Rust-compiled to wasm file 
    - a JavaScript wrapper file around the wasm
    - TypeScript declaration files to convey information about types
    - a `package.json` file
