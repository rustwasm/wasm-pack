use PBAR;
use bindgen;
use build;
use console::style;
use emoji;
use failure::Error;
use indicatif::HumanDuration;
use manifest;
use npm;
use quicli::prelude::*;
use readme;
use std::fs;
use std::result;
use std::time::Instant;

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

pub fn init(path: Option<String>, scope: Option<String>) -> result::Result<(), Error> {
    let started = Instant::now();

    let crate_path = set_crate_path(path);

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

pub fn pack(path: Option<String>) -> result::Result<(), Error> {
    let crate_path = set_crate_path(path);

    npm::npm_pack(&crate_path);
    println!("🎒  packed up your package!");
    Ok(())
}

pub fn publish(path: Option<String>) -> result::Result<(), Error> {
    let crate_path = set_crate_path(path);

    npm::npm_publish(&crate_path);
    println!("💥  published your package!");
    Ok(())
}

fn set_crate_path(path: Option<String>) -> String {
    let crate_path = match path {
        Some(p) => p,
        None => ".".to_string(),
    };

    crate_path
}
