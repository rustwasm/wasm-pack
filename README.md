# 📦✨  wasm-pack
> pack up the wasm and publish it to npm!

[![Build Status](https://travis-ci.org/ashleygwilliams/wasm-pack.svg?branch=master)](https://travis-ci.org/ashleygwilliams/wasm-pack)

the goal of this project is to create a portable command line tool
for publishing compiled wasm projects to the npm registry for the consumption
of js devs using the npm CLI, yarn, or any other CLI tool that interfaces
with the npm registry.

this project is a part of the [rust-wasm] group. you can find more info by
visiting that repo!

[rust-wasm]: https://github.com/rust-lang-nursery/rust-wasm/

## 🔮 prerequisities

this project is written in rust. [get rust] to work on this project.

[get rust]: https://www.rustup.rs/

## 🏃‍♀️ up and running

1. fork and clone this repository
2. `cd wasm-pack`
3. `cargo run`

## ⚙️ what's it do?

right now? not much. here's the plan:

- [x] read data from `Cargo.toml`
- [ ] read JS dependency data from your compiled wasm (see [rust-wasm/36])
- [x] write data to `package.json`
- [ ] log you in to npm
- [ ] publish package to npm registry

[rust-wasm/36]: https://github.com/rust-lang-nursery/rust-wasm/issues/36
