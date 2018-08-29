extern crate copy_dir;
extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate structopt;
extern crate tempfile;
extern crate wasm_pack;
#[macro_use]
extern crate lazy_static;

mod build;
mod manifest;
mod readme;
mod utils;
