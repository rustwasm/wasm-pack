extern crate failure;
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
    let path_to_pkg_dir = format!("{}/pkg", path);
    fs::create_dir_all(path_to_pkg_dir)?;
    Ok(())
}
