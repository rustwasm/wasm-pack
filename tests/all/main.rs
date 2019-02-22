extern crate assert_cmd;
extern crate failure;
extern crate predicates;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate binary_install;
extern crate serde_json;
extern crate structopt;
extern crate tempfile;
extern crate wasm_pack;

mod bindgen;
mod build;
mod license;
mod lockfile;
mod manifest;
mod readme;
mod test;
mod utils;
mod webdriver;
