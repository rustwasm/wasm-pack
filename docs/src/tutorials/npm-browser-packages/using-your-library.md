# Run The Code From npm

This portion of the tutorial will help you create a [Webpack] JavaScript project that will
run your WebAssembly code in the browser.

[Webpack]: https://webpack.js.org/

## Scaffold a JavaScript Project

To scaffold a project that we can use our new package in, we'll use an npm template called
[`create-wasm-app`]. To use this run this command in a directory *different* than your Rust
project:

[`create-wasm-app`]: https://github.com/rustwasm/create-wasm-app

```
npm init wasm-app my-new-wasm-app
```

Instead of `my-new-wasm-app` you can choose a different project name.
The tool will create a directory with that name.

If we look in that directory, we'll see the following:

- `.gitignore`: ignores `node_modules`
- `LICENSE-APACHE` and `LICENSE-MIT`: most Rust projects are licensed this way, so these are included for you
- `README.md`: the file you are reading now!
- `index.html`: a bare bones html document that includes the webpack bundle
- `index.js`: example js file with a comment showing how to import and use a wasm pkg
- `package.json` and `package-lock.json`: 
  - pulls in devDependencies for using webpack:
      - [`webpack`](https://www.npmjs.com/package/webpack)
      - [`webpack-cli`](https://www.npmjs.com/package/webpack-cli)
      - [`webpack-dev-server`](https://www.npmjs.com/package/webpack-dev-server)
  - defines a `start` script to run `webpack-dev-server`
- `webpack.config.js`: configuration file for bundling your js with webpack

## Add Your npm Package

The scaffolded project includes an example WebAssembly package, `hello-wasm-pack`.
This package is listed in the `dependencies` array in your `package.json` file.
Its files are stored in `node_packages/hello-wasm-pack`.

Use `npm` to remove the package:

```
npm remove hello-wasm-pack
```

Now add your compiled WebAssembly package:

To add a local package:

```
npm link ../my-wasm-pack-package/pkg/
```

To add a published package from NPM servers:

```
npm install my-wasm-package
```

Now open the `index.js` file.
Replace the `hello-wasm-pack` in the first line with the name of your package:

```js
import * as wasm from "<your package name>";

wasm.greet();
```

## Run The Project

Before we run our project, we need to make sure we install our dependencies:

```bash
npm install
```

We should be ready to run our project now! To run our project we'll run:

```bash
npm start
```

Then in a web browser navigate to `http://localhost:8080` and you should be greeted with an
alert box that says "Hello World!".

If you did congrats you've successfully uploaded your first bit of wasm code to npm and used it
properly!
