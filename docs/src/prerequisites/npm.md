# npm

Currently, `wasm-pack` requires that you have npm installed to pack and publish your
package. Longterm, this will be replaced by a Rust only version.

If you would rather use another package manager that interfaces with the npm registry
you may, however, the `pack` and `publish` commands depend on having npm installed.

You can install [npm] by following [these instructions][npm-install-info].

### npm Account

Regardless of which package manager CLI tool you'l like to you, if you wish to publish
your package to the npm registry, or another npm-like registry, you'll need an npm
account.

You can find information about signing up for npm [here][npm-signup-info].

[npm-install-info]: https://www.npmjs.com/get-npm
[npm-signup-info]: https://www.npmjs.com/signup
