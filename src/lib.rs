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

use std::fs;

use failure::Error;

pub fn create_pkg_dir(path: &str) -> Result<(), Error> {
    let pkg_dir_path = format!("{}/pkg", path);
    fs::create_dir_all(pkg_dir_path)?;
    Ok(())
}
