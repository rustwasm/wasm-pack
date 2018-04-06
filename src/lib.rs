extern crate console;
extern crate failure;
extern crate indicatif;
#[macro_use]
extern crate lazy_static;
extern crate quicli;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

pub mod bindgen;
pub mod build;
pub mod emoji;
pub mod manifest;
pub mod npm;
pub mod progressbar;
pub mod readme;

use quicli::prelude::*;
use std::fs;
use std::time::Instant;

use console::style;
use failure::Error;
use indicatif::HumanDuration;
use progressbar::ProgressOutput;

lazy_static! {
    pub static ref PBAR: ProgressOutput = { ProgressOutput::new() };
}

// quicli::prelude::* imports a different result struct which gets
// precedence over the std::result::Result, so have had to specify
// the correct type here.
pub fn create_pkg_dir(path: &str) -> std::result::Result<(), Error> {
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

pub fn init_command(path: Option<String>, scope: std::option::Option<String>) -> std::result::Result<(), Error> {
    let started = Instant::now();

    let crate_path = match path {
        Some(p) => p,
        None => ".".to_string(),
    };

    build::rustup_add_wasm_target();
    build::cargo_build_wasm(&crate_path);
    create_pkg_dir(&crate_path)?;
    manifest::write_package_json(&crate_path, scope)?;
    readme::copy_from_crate(&crate_path)?;
    bindgen::cargo_install_wasm_bindgen();
    let name = manifest::get_crate_name(&crate_path)?;
    bindgen::wasm_bindgen_build(&crate_path, &name);
    PBAR.one_off_message(&format!(
            "{} Done in {}",
            emoji::SPARKLE,
            HumanDuration(started.elapsed())
    ));
    PBAR.one_off_message(&format!(
            "{} Your WASM pkg is ready to publish at {}/pkg",
            emoji::PACKAGE,
            &crate_path
    ));
    PBAR.done()?;
    Ok(())
}

pub fn pack_command(path: Option<String>) -> std::result::Result<(),Error> {
    let crate_path = match path {
        Some(p) => p,
        None => ".".to_string(),
    };
    npm::npm_pack(&crate_path);
    println!("🎒  packed up your package!");
    Ok(())
}

pub fn publish_command(path: Option<String>) -> std::result::Result<(), Error> {
    let crate_path = match path {
        Some(p) => p,
        None => ".".to_string(),
    };
    npm::npm_publish(&crate_path);
    println!("💥  published your package!");
    Ok(())
}

/// 📦 ✨  pack and publish your wasm!
#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    pub cmd: Command,
    ///  log all the things
    #[structopt(long = "verbose", short = "v", parse(from_occurrences))]
    pub verbosity: u8,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "init")]
    /// 🐣  initialize a package.json based on your cmpiled wasm
    Init {
        path: Option<String>,
        #[structopt(long = "scope", short = "s")]
        scope: Option<String>,
    },
    #[structopt(name = "pack")]
    /// 🍱  create a tar of your npm package but don't ublish! [NOT IMPLEMENTED]
    Pack { path: Option<String> },
    #[structopt(name = "publish")]
    /// 🎆  pack up your npm package and publish! [NOT MPLEMENTED]
    Publish { path: Option<String> },
}
