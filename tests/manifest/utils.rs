use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use failure::Error;
use serde_json;

pub trait FailureExt<T> {
    fn unwrap_pretty(self) -> T;
}

impl<T> FailureExt<T> for Result<T, Error> {
    fn unwrap_pretty(self) -> T {
        self.unwrap_or_else(|e| {
            for e in e.causes() {
                println!("err: {}", e);
            }
            panic!("unwrap failed");
        })
    }
}

#[derive(Deserialize)]
pub struct NpmPackage {
    pub name: String,
    pub description: String,
    pub version: String,
    pub license: String,
    pub repository: Repository,
    pub files: Vec<String>,
    pub main: String,
    pub dependencies: Option<BTreeMap<String, String>>,
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

const WASM: &[u8] = include_bytes!("add.wasm");

pub fn mock_wasm(path: &Path, name: &str) {
    let path = path.join("pkg").join(format!("{}_bg.wasm", name));
    File::create(&path).unwrap().write_all(WASM).unwrap();
}

const DEP_WASM: &[u8] = include_bytes!("dependencies.wasm");

pub fn mock_wasm_with_deps(path: &Path, name: &str) {
    let path = path.join("pkg").join(format!("{}_bg.wasm", name));
    File::create(&path).unwrap().write_all(DEP_WASM).unwrap();
}
