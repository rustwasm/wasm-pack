extern crate console;
extern crate failure;
extern crate indicatif;
#[macro_use]
extern crate lazy_static;
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

use std::fs;

use console::style;
use failure::Error;
use progressbar::ProgressOutput;

lazy_static! {
    pub static ref PBAR: ProgressOutput = { ProgressOutput::new() };
}

pub fn create_pkg_dir(path: &str) -> Result<(), Error> {
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
