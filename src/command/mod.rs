use error::Error;
use npm;
#[allow(unused)]
use quicli::prelude::*;
use std::result;

mod build;
mod install;

pub use self::build::cargo_build_wasm;
pub use self::install::{cargo_install_wasm_bindgen, rustup_add_wasm_target};

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "init")]
    /// 🐣  initialize a package.json based on your compiled wasm!
    Init {
        path: Option<String>,
        #[structopt(long = "scope", short = "s")]
        scope: Option<String>,
    },
    #[structopt(name = "pack")]
    /// 🍱  create a tar of your npm package but don't publish!
    Pack { path: Option<String> },
    #[structopt(name = "publish")]
    /// 🎆  pack up your npm package and publish!
    Publish { path: Option<String> },
}
