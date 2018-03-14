use std::fs::File;
use std::io::prelude::*;

use console::style;
use failure::Error;
use serde_json;
use toml;
use emoji;
use progressbar;

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
    let mut cargo_file = File::open(manifest_path)?;
    let mut cargo_contents = String::new();
    cargo_file.read_to_string(&mut cargo_contents)?;

    Ok(toml::from_str(&cargo_contents)?)
}

impl CargoManifest {
    fn into_npm(mut self, scope: Option<String>) -> NpmPackage {
        let filename = self.package.name.replace("-", "_");
        let js_file = format!("{}.js", filename);
        let wasm_file = format!("{}_bg.wasm", filename);
        if let Some(s) = scope {
            self.package.name = format!("@{}/{}", s, self.package.name);
        }
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
pub fn write_package_json(path: &str, scope: Option<String>) -> Result<(), Error> {
    let step = format!(
        "{} {}Writing a package.json...",
        style("[4/7]").bold().dim(),
        emoji::MEMO
    );
    let pb = progressbar::new(step);
    let pkg_file_path = format!("{}/pkg/package.json", path);
    let mut pkg_file = File::create(pkg_file_path)?;
    let crate_data = read_cargo_toml(path)?;
    let npm_data = crate_data.into_npm(scope);
    let npm_json = serde_json::to_string(&npm_data)?;
    pkg_file.write_all(npm_json.as_bytes())?;
    pb.finish();
    Ok(())
}

pub fn get_crate_name(path: &str) -> Result<String, Error> {
    Ok(read_cargo_toml(path)?.package.name)
}
