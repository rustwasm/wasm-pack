use console::style;
use emoji;
use error::Error;
use npm;
#[allow(unused)]
use quicli::prelude::*;
use std::fs;
use std::result;
use PBAR;

mod build;
mod init;

pub use self::build::{cargo_build_wasm, rustup_add_wasm_target};
pub use self::init::init;

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

// quicli::prelude::* imports a different result struct which gets
// precedence over the std::result::Result, so have had to specify
// the correct type here.
pub fn create_pkg_dir(path: &str) -> result::Result<(), Error> {
    let step = format!(
        "{} {}Creating a pkg directory...",
        style("[3/7]").bold().dim(),
        emoji::FOLDER
    );
    let pb = PBAR.message(&step);
    let pkg_dir_path = format!("{}/pkg", path);
    fs::create_dir_all(pkg_dir_path)?;
    pb.finish();
    Ok(())
}

pub fn pack(crate_path: &str) -> result::Result<(), Error> {
    npm::npm_pack(&crate_path)?;
    PBAR.message("🎒  packed up your package!");
    Ok(())
}

pub fn publish(crate_path: &str) -> result::Result<(), Error> {
    npm::npm_publish(&crate_path)?;
    PBAR.message("💥  published your package!");
    Ok(())
}
