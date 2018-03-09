use std::fs::File;
use std::io::prelude::*;

use failure::Error;
use serde_json;
use toml;

#[derive(Deserialize)]
struct CargoManifest {
    package: CargoPackage,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
    description: String,
    version: String,
    license: String,
    repository: String,
}

#[derive(Serialize)]
struct NpmPackage {
    name: String,
    description: String,
    version: String,
    license: String,
    repository: Repository,
    files: Vec<String>,
}

#[derive(Serialize)]
struct Repository {
    #[serde(rename = "type")]
    ty: String,
    url: String,
}

fn read_cargo_toml(path: &str) -> Result<CargoManifest, Error> {
    let manifest_path = format!("{}/Cargo.toml", path);
    println!("👩<200d>🍳  reading {}", manifest_path);
    let mut cargo_file = File::open(manifest_path)?;
    let mut cargo_contents = String::new();
    cargo_file.read_to_string(&mut cargo_contents)?;

    Ok(toml::from_str(&cargo_contents)?)
}

impl CargoManifest {
    fn into_npm(self) -> NpmPackage {
        let filename = self.package.name.replace("-", "_");
        let js_file = format!("{}.js", filename);
        let wasm_file = format!("{}_bg.wasm", filename);
        NpmPackage {
            name: self.package.name,
            description: self.package.description,
            version: self.package.version,
            license: self.package.license,
            repository: Repository {
                ty: "git".to_string(),
                url: self.package.repository,
            },
            files: vec![js_file, wasm_file],
        }
    }
}

/// Generate a package.json file inside in `./pkg`.
pub fn write_package_json(path: &str) -> Result<(), Error> {
    let pkg_file_path = format!("{}/pkg/package.json", path);
    let mut pkg_file = File::create(pkg_file_path)?;
    let crate_data = read_cargo_toml(path)?;
    let npm_data = crate_data.into_npm();
    let npm_json = serde_json::to_string(&npm_data)?;
    pkg_file.write_all(npm_json.as_bytes())?;
    println!("✍️  wrote a package.json!");
    Ok(())
}

pub fn get_crate_name(path: &str) -> Result<String, Error> {
    Ok(read_cargo_toml(path)?.package.name)
}
