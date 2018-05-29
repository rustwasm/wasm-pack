# Changelog

## 💫 0.3.0

### ✨ Features

- **Logging - [mgattozzi], [pull/134]**

  Up until now, we've forced folks to rely on emoji-jammed console output to debug
  errors. While emojis are fun, this is often not the most pleasant experience. Now
  we'll generate a `wasm-pack.log` file if `wasm-pack` errors on you, and you can
  customize the log verbosity using the (previously unimplemented) verbosity flag.

[pull/134]: https://github.com/ashleygwilliams/wasm-pack/pull/134

- **`--target` flag - [djfarly], [pull/132]**

  `wasm-bindgen-cli` is able to generate a JS module wrapper for generated wasm files
  for both ES6 modules and CommonJS. Up until now, we only used wasm-bindgen's default
  behavior, ES6 modules. You can now pass a `--target` flag with either `nodejs` or 
  `browser` to generate the type of module you want to use. Defaults to `browser` if not
  passed.

[djfarly]: https://github.com/djfarly
[pull/132]: https://github.com/ashleygwilliams/wasm-pack/pull/132

- **human readable panics - [yoshuawuyts], [pull/118]**

  Panics aren't always the most friendly situation ever. While we never want to panic on ya,
  if we do- we'll do it in a way that's a little more readable now.

[pull/118]: https://github.com/ashleygwilliams/wasm-pack/pull/118

- **typescript support by default - [kwonoj], [pull/109]**

  `wasm-bindgen` now generates typescript type files by default. To suppress generating
  the type file you can pass the `--no-typescript` flag. The type file is useful for more
  than just typescript folks- many IDEs use it for completion!

[kwonoj]: https://github.com/kwonoj
[pull/109]: https://github.com/ashleygwilliams/wasm-pack/pull/109

- **wrap `npm login` command - [djfarly], [pull/100]**

  In order to publish a package to npm, you need to be logged in. You can now use
  `wasm-pack login` to login to the npm (or any other) registry.

[pull/100]: https://github.com/ashleygwilliams/wasm-pack/pull/100

- **exit early on failure - [mgattozzi], [pull/90]**

  Until now, `wasm-pack` would continue to run tasks, even if a task failed. Now- if something
  fails, we'll exit so you don't have to wait to fix the error.

[pull/90]: https://github.com/ashleygwilliams/wasm-pack/pull/90

### 🤕 Fixes

- **force install wasm-bindgen - [ashleygwilliams], [pull/133]**

  Using an out of date version of `wasm-bindgen` can run you into a bunch of trouble. This
  very small change should fix the large number of bug reports we received from users using
  an out of date `wasm-bindgen-cli` by force installing `wasm-bindgen-cli` to ensure the user
  always has the latest version. We don't expect this to be a forever solution (it's a bit
  slow!) but it should help those who are getting started have a less rough time.

[pull/133]: https://github.com/ashleygwilliams/wasm-pack/pull/133

- **fix CI release builds - [ashleygwilliams], [pull/135]**

  This was not working! But now it is! You can always use `cargo install` to install
  wasm-pack, but now you can find pre-built Linux and Mac binaries in the [Releases]
  tab of our GitHub repo.

[Releases]: https://github.com/ashleygwilliams/wasm-pack/releases
[pull/135]: https://github.com/ashleygwilliams/wasm-pack/pull/135   

### 🛠️ Maintenance 

- **remove `quicli` dependency - [mgattozzi], [pull/131]**

  While `quicli` is a great way to get started writing a CLI app in Rust- it's not meant for 
  large, mature applications. Now that `wasm-pack` is bigger and has many active users, we've
  removed this dependency to unblock further development on the tool.

[pull/131]: https://github.com/ashleygwilliams/wasm-pack/pull/131

- **update rustfmt CI test - [djfarly], [pull/128]**

  Since 0.2.0 how one should call `rustfmt` changed! We've kept it up to date so we can continue
  to maintain conventional style in the codebase.

[pull/128]: https://github.com/ashleygwilliams/wasm-pack/pull/128

- **custom module for errors - [mgattozzi], [pull/120]**

  Thanks to the `failure` crate, we've been playing fast and loose with errors for a bit. We're
  finally getting serious about error handling - by organizing all of our specific errors in a
  specific module. This will make it easier to communicate these errors out and handle new error
  cases from future features.

[pull/120]: https://github.com/ashleygwilliams/wasm-pack/pull/120

### 📖 Documentation

Special thanks to [data-pup] who continues to be our documentation champion! In case you missed it,
check out the guides in the [docs directory!](docs)!

## 🌌 0.2.0

This release focuses on filling out all commands and improving stderr/out
handling for improved user experience!

### ✨ Features

- **`pack` and `publish` - [jamiebuilds], [pull/67]**
  You can now run `wasm-pack pack` to generate a tarball of your generated package,
  as well as run `wasm-pack publish` to publish your package to the npm registry.
  Both commands require that you have npm installed, and the `publish` command requires
  that you be logged in to the npm client. We're working on wrapping the `npm login`
  command so that you can also login directly from `wasm-pack`, see [pull/100] for more
  details.

[jamiebuilds]: https://github.com/jamiebuilds
[pull/67]: https://github.com/ashleygwilliams/wasm-pack/pull/67
[pull/100]: https://github.com/ashleygwilliams/wasm-pack/pull/100

- **`package.json` is pretty printed now - [yoshuawuyts], [pull/70]**

  Previously, `package.json` was not very human readable. Now it is pretty printed!

- **`collaborators` - [yoshuawuyts], [pull/70]**

  `wasm-pack` now will fill out the `collaborators` field in your `package.json` for
  you based on your `Cargo.toml` `authors` data. For more discussion on how we decided
  on this v.s. other types of `author` fields in `package.json`, see [issues/2].

[yoshuawuyts]: https://github.com/yoshuawuyts
[pull/70]: https://github.com/ashleygwilliams/wasm-pack/pull/70
[issues/2]: https://github.com/ashleygwilliams/wasm-pack/issues/2

- **Release binaries built with CI - [ashleygwilliams], [pull/103]**

[ashleygwilliams]: https://github.com/ashleygwilliams
[pull/103]: https://github.com/ashleygwilliams/wasm-pack/pull/103

### 🤕 Fixes

- **Optional `package.json` fields warn instead of failing - [mgattozzi], [pull/65]**

[pull/65]: https://github.com/ashleygwilliams/wasm-pack/pull/65

- **Program doesn't swallow stout and sterr - [mgattozzi], [pull/90]**

[mgattozzi]: https://github.com/mgattozzi
[pull/90]: https://github.com/ashleygwilliams/wasm-pack/pull/90

### 🛠️ Maintenance and 📖 Documentation

Thanks so much to [mgattozzi], [data-pup], [sendilkumarn], [Andy-Bell], 
[steveklabnik], [jasondavies], and [edsrzf] for all the awesome refactoring,
documentation, typo-fixing, and testing work. We appreciate it so much!

[data-pup]: https://github.com/data-pup
[sendilkumarn]: https://github.com/sendilkumarn
[Andy-Bell]: https://github.com/Andy-Bell
[steveklabnik]: https://github.com/steveklabnik
[jasondavies]: https://github.com/jasondavies
[edsrzf]: https://github.com/edsrzf

## 💥  0.1.0

- First release! 
