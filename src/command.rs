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

pub fn run_wasm_pack(command: Command) -> result::Result<(), Error> {
    // Run the correct command based off input and store the result of it so that we can clear
    // the progress bar then return it
    let status = match command {
        Command::Init { path, scope } => init(path, scope),
        Command::Pack { path } => pack(path),
        Command::Publish { path } => publish(path),
    };

    match status {
        Ok(_) => {}
        Err(ref e) => {
            PBAR.error(e.error_type());
        }
    }

    // Make sure we always clear the progress bar before we abort the program otherwise
    // stderr and stdout output get eaten up and nothing will work. If this part fails
    // to work and clear the progress bars then you're really having a bad day with your tools.
    PBAR.done()?;

    // Return the actual status of the program to the main function
    status
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

fn init(path: Option<String>, scope: Option<String>) -> result::Result<(), Error> {
    let started = Instant::now();

    let crate_path = set_crate_path(path);

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

fn pack(path: Option<String>) -> result::Result<(), Error> {
    let crate_path = set_crate_path(path);

    npm::npm_pack(&crate_path)?;
    PBAR.message("🎒  packed up your package!");
    Ok(())
}

fn publish(path: Option<String>) -> result::Result<(), Error> {
    let crate_path = set_crate_path(path);

    npm::npm_publish(&crate_path)?;
    PBAR.message("💥  published your package!");
    Ok(())
}

fn set_crate_path(path: Option<String>) -> String {
    let crate_path = match path {
        Some(p) => p,
        None => ".".to_string(),
    };

    crate_path
}
