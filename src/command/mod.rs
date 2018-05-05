use bindgen;
use build;
use console::style;
use emoji;
use error::Error;
use indicatif::HumanDuration;
use manifest;
use npm;
#[allow(unused)]
use quicli::prelude::*;
use readme;
use std::fs;
use std::result;
use std::time::Instant;
use Cli;
use PBAR;

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

pub fn init(crate_path: String, scope: Option<String>) -> result::Result<(), Error> {
    let started = Instant::now();

    build::rustup_add_wasm_target()?;
    build::cargo_build_wasm(&crate_path)?;
    create_pkg_dir(&crate_path)?;
    manifest::write_package_json(&crate_path, scope)?;
    readme::copy_from_crate(&crate_path)?;
    bindgen::cargo_install_wasm_bindgen()?;
    let name = manifest::get_crate_name(&crate_path)?;
    bindgen::wasm_bindgen_build(&crate_path, &name)?;
    PBAR.message(&format!(
        "{} Done in {}",
        emoji::SPARKLE,
        HumanDuration(started.elapsed())
    ));
    PBAR.message(&format!(
        "{} Your WASM pkg is ready to publish at {}/pkg",
        emoji::PACKAGE,
        &crate_path
    ));
    Ok(())
}

pub fn pack(crate_path: String) -> result::Result<(), Error> {
    npm::npm_pack(&crate_path)?;
    PBAR.message("🎒  packed up your package!");
    Ok(())
}

pub fn publish(crate_path: String) -> result::Result<(), Error> {
    npm::npm_publish(&crate_path)?;
    PBAR.message("💥  published your package!");
    Ok(())
}
