extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate structopt;
extern crate tempfile;
extern crate wasm_pack;
extern crate wasm_pack_binary_install;

mod bindgen;
mod build;
mod license;
mod lockfile;
mod manifest;
mod readme;
mod test;
mod utils;
mod webdriver;
