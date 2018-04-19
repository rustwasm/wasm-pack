use std::fs::File;
use std::io::prelude::*;

use failure::Error;
use serde_json;

#[derive(Deserialize)]
pub struct NpmPackage {
    pub name: String,
    pub description: String,
    pub version: String,
    pub license: String,
    pub repository: Repository,
    pub files: Vec<String>,
    pub main: String,
}

#[derive(Deserialize)]
pub struct Repository {
    #[serde(rename = "type")]
    pub ty: String,
    pub url: String,
}

pub fn read_package_json(path: &str) -> Result<NpmPackage, Error> {
    let manifest_path = format!("{}/pkg/package.json", path);
    let mut pkg_file = File::open(manifest_path)?;
    let mut pkg_contents = String::new();
    pkg_file.read_to_string(&mut pkg_contents)?;

    Ok(serde_json::from_str(&pkg_contents)?)
}
