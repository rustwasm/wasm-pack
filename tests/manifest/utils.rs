use std::fs::File;
use std::io::prelude::*;

use failure::Error;
use serde_json;
use toml;

use wasm_pack::Cli;
use wasm_pack::command::Command;
use wasm_pack::context::{Action, Context};
use wasm_pack::context::progressbar::ProgressOutput;
use wasm_pack::manifest::{CargoManifest, NpmPackage};

pub fn read_package_json(path: &str) -> Result<NpmPackage, Error> {
    let manifest_path = format!("{}/pkg/package.json", path);
    let mut pkg_file = File::open(manifest_path)?;
    let mut pkg_contents = String::new();
    pkg_file.read_to_string(&mut pkg_contents)?;

    Ok(serde_json::from_str(&pkg_contents)?)
}

pub fn get_crate_name(path: &str) -> String {
    let manifest = get_crate_manifest(&path);
    manifest.package.name.clone()
}

pub fn get_crate_manifest(path: &str) -> CargoManifest {
    // FIXUP: Not a big fan of this function. How to expose in the Context struct?
    let manifest_path = format!("{}/Cargo.toml", path);
    let mut cargo_file = File::open(manifest_path).unwrap();
    let mut cargo_contents = String::new();
    cargo_file.read_to_string(&mut cargo_contents).unwrap();
    toml::from_str(&cargo_contents).unwrap()
}
