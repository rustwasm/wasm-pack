#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;
extern crate failure;

use std::fs::File;
use std::io::prelude::*;

use failure::Error;

#[derive(Deserialize)]
struct CargoManifest {
    package: CargoPackage,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
    description: String,
    version: String,
}

#[derive(Serialize)]
struct NpmPackage {
    name: String,
    description: String,
    version: String,
}

fn read_cargo_toml() -> Result<CargoManifest, Error> {
    let mut cargo_file = File::open("Cargo.toml")?;
    let mut cargo_contents = String::new();
    cargo_file.read_to_string(&mut cargo_contents)?;

    Ok(toml::from_str(&cargo_contents)?)
}

impl CargoManifest {
    fn into_npm(self) -> NpmPackage {
        NpmPackage {
            name: self.package.name,
            description: self.package.description,
            version: self.package.version,
        }
    }
}

pub fn write_package_json() -> Result<(), Error> {
    let mut pkg_file = File::create("package.json")?;
    let crate_data = read_cargo_toml()?;
    let npm_data = crate_data.into_npm();
    let npm_json = serde_json::to_string(&npm_data)?;
    pkg_file.write_all(npm_json.as_bytes())?;
    Ok(())
}
