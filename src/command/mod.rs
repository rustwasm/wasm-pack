#[allow(unused)]
use quicli::prelude::*;

mod bindgen;
mod build;
mod install;
mod npm;

pub use self::bindgen::wasm_bindgen_build;
pub use self::build::cargo_build_wasm;
pub use self::install::{cargo_install_wasm_bindgen, rustup_add_wasm_target};
pub use self::npm::{npm_pack, npm_publish};

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
