use std::fs::File;
use std::io::prelude::*;

use error::Error;
use toml;

mod cargo;
mod npm;

pub use self::cargo::{CargoManifest, CargoPackage};
pub use self::npm::{NpmPackage, Repository};

/// Try to read the `Cargo.toml` file in the given directory.
pub fn read_cargo_toml(path: &str) -> Result<CargoManifest, Error> {
    let manifest_path = format!("{}/Cargo.toml", path);
    let mut cargo_file = File::open(manifest_path)?;
    let mut cargo_contents = String::new();
    cargo_file.read_to_string(&mut cargo_contents)?;

    Ok(toml::from_str(&cargo_contents)?)
}
