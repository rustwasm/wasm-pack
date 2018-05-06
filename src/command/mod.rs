use console::style;
use emoji;
use error::Error;
use npm;
#[allow(unused)]
use quicli::prelude::*;
use std::fs;
use std::result;

mod build;
mod install;

pub use self::build::cargo_build_wasm;
// pub use self::init::init;
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

// FIXUP: Both of these functions can be moved perhaps in their entirety
// into the Context wrapper methods. The npm module is doing the heavy lifting
// here. (Context is ideally for these PBAR messages anyhow!)

pub fn pack(crate_path: &str) -> result::Result<(), Error> {
    npm::npm_pack(&crate_path)?;
    // PBAR.message("🎒  packed up your package!"); // FIXUP
    Ok(())
}

pub fn publish(crate_path: &str) -> result::Result<(), Error> {
    npm::npm_publish(&crate_path)?;
    // PBAR.message("💥  published your package!"); // FIXUP
    Ok(())
}
