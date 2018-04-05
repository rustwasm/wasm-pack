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
use std::fs::File;
use std::io::Read;

use console::style;
use failure::Error;
use progressbar::ProgressOutput;

fn read_cargo_toml_to_string() -> String {
    let mut cargo_file = File::open("Cargo.toml").expect("Unable to open Cargo.toml");
    let mut cargo_contents = String::new();
    cargo_file.read_to_string(&mut cargo_contents).expect("Failed to read Cargo.toml");

    return cargo_contents;
}

lazy_static! {
    pub static ref PBAR: ProgressOutput = { ProgressOutput::new() };
    
    pub static ref CARGO_TOML: String = read_cargo_toml_to_string();
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
