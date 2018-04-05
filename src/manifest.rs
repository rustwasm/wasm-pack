use std::fs::File;
use std::io::prelude::*;

use PBAR;
use CARGO_TOML;
use console::style;
use emoji;
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
    authors: Vec<String>,
    description: Option<String>,
    version: String,
    license: Option<String>,
    repository: Option<String>,
}

#[derive(Serialize)]
struct NpmPackage {
    name: String,
    collaborators: Vec<String>,
    description: Option<String>,
    version: String,
    license: Option<String>,
    repository: Option<Repository>,
    files: Vec<String>,
}

#[derive(Serialize)]
struct Repository {
    #[serde(rename = "type")]
    ty: String,
    url: String,
}

// fn read_cargo_toml(path: &str) -> Result<CargoManifest, Error> {
//     let manifest_path = format!("{}/Cargo.toml", path);
//     let mut cargo_file = File::open(manifest_path)?;
//     let mut cargo_contents = String::new();
//     cargo_file.read_to_string(&mut cargo_contents)?;

//     Ok(toml::from_str(&cargo_contents)?)
// }

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
            collaborators: self.package.authors,
            description: self.package.description,
            version: self.package.version,
            license: self.package.license,
            repository: self.package.repository.map(|repo_url| Repository {
                ty: "git".to_string(),
                url: repo_url,
            }),
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

    let warn_fmt = |field| {
        format!(
            "Field {} is missing from Cargo.toml. It is not necessary, but recommended",
            field
        )
    };

    let pb = PBAR.message(&step);
    let pkg_file_path = format!("{}/pkg/package.json", path);
    let mut pkg_file = File::create(pkg_file_path)?;
    let crate_data: CargoManifest = toml::from_str(&CARGO_TOML.to_string())?;
    let npm_data = crate_data.into_npm(scope);

    if npm_data.description.is_none() {
        PBAR.warn(&warn_fmt("description"));
    }
    if npm_data.repository.is_none() {
        PBAR.warn(&warn_fmt("repository"));
    }
    if npm_data.license.is_none() {
        PBAR.warn(&warn_fmt("license"));
    }

    let npm_json = serde_json::to_string_pretty(&npm_data)?;
    pkg_file.write_all(npm_json.as_bytes())?;
    pb.finish();
    Ok(())
}

pub fn get_crate_name() -> Result<String, Error> {
    let crate_data: CargoManifest = toml::from_str(&CARGO_TOML.to_string())?;
    Ok(crate_data.package.name)
}
