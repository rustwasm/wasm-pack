extern crate console;
extern crate failure;
extern crate indicatif;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

pub mod build;
pub mod bindgen;
pub mod readme;
pub mod manifest;
pub mod progressbar;
pub mod emoji;

use std::fs;

use failure::Error;
use console::style;

pub fn create_pkg_dir(path: &str) -> Result<(), Error> {
    let step = format!(
        "{} {}Creating a pkg directory...",
        style("[3/7]").bold().dim(),
        emoji::FOLDER
    );
    let pb = progressbar::new(step);
    let pkg_dir_path = format!("{}/pkg", path);
    fs::create_dir_all(pkg_dir_path)?;
    pb.finish();
    Ok(())
}
