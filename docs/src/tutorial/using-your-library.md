# Run The Code From npm

Alright let's make a new small directory to test that we can now run this code and pull it from npm.

```bash
$ mkdir test
$ cd test
```

Now we need to create a `package.json` file that looks like this:

```json
{
  "scripts": {
    "serve": "webpack-dev-server"
  },
  "dependencies": {
    "@MYSCOPE/wasm-add": "^0.1.0"
  },
  "devDependencies": {
    "webpack": "^4.0.1",
    "webpack-cli": "^2.0.10",
    "webpack-dev-server": "^3.1.0"
  }
}
```

where `MYSCOPE` is your npm username. You can expand this to be a more complete file but
we're really just trying to verify that this works!

Next up we'll need to create a small webpack configuration so that we can use the
`webpack-dev-server` to serve the wasm file properly. It should be noted that webpack isn't
a requirement. It's just what was chosen for this tutorial. You just need something to server the
code! Here's what your `webpack.config.js` should look like:

```javascript
const path = require('path');
module.exports = {
  entry: "./index.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "index.js",
  },
  mode: "development"
};
```

This tells webpack that if it's going to start things up use `index.js`. Before we do that though
we'll need to setup a small html file. Create a new file called `index.html` and put this inside it:

```html
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <title>wasm-pack example</title>
  </head>
  <body>
    <script src="./index.js"></script>
  </body>
</html>
```

We're almost set. Now we need to setup our JS file so that we can run some wasm code!
Make a file called `index.js` and put this inside of it:

```javascript
const js = import("@MYSCOPE/wasm-add/wasm_add.js");
js.then(js => {
  js.alert_add(3,2);
});
```

Since web pack [can't load wasm synchronously yet](https://github.com/webpack/webpack/issues/6615)
we are using the import statement above followed
by the promise in order to load it properly. This is what lets us then call `alert_add`. We're
importing from the `node_module` folder we haven't gotten yet so let's import all of our
dependencies finally and run the example!

```bash
$ npm install
$ npm run serve
```

Then in a web browser navigate to `http://localhost:8080` you should see something like this:

![An alert box saying "Hello from Rust! 3 + 2 = 5"](./wasm-pack/wasm-pack.png)

If you did congrats you've successfully uploaded your first bit of wasm code to npm and used it
properly!
