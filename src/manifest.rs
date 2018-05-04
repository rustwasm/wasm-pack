use std::fs::File;
use std::io::prelude::*;

use console::style;
use emoji;
use error::Error;
use serde_json;
use toml;
use PBAR;

lazy_static! {
    pub static ref CARGO_TOML: CargoManifest = read_cargo_toml(".").unwrap();
}

#[derive(Deserialize)]
pub struct CargoManifest {
    package: CargoPackage,
}

impl CargoManifest {
    pub fn get_crate_name(&self) -> String {
        self.package.name.clone()
    }
}

#[derive(Deserialize)]
pub struct CargoPackage {
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
    main: String,
}

impl NpmPackage {
    /// Create an NpmPackage from a borrowed `CargoManifest` object.
    pub fn from_manifest(cargo: &CargoManifest) -> NpmPackage {
        let (name, description) = NpmPackage::get_package_name_and_desc(cargo);
        let (js_file, wasm_file) = NpmPackage::get_js_and_wasm_filenames(cargo);
        let repository = NpmPackage::get_repo(cargo);
        NpmPackage {
            name,
            collaborators: cargo.package.authors.clone(),
            description,
            version: cargo.package.version.clone(),
            license: cargo.package.license.clone(),
            repository,
            files: vec![wasm_file],
            main: js_file,
        }
    }

    fn get_package_name_and_desc(cargo: &CargoManifest) -> (String, Option<String>) {
        let package_name = cargo.package.name.clone();
        let description = cargo.package.description.clone();
        (package_name, description)
    }

    fn get_repo(cargo: &CargoManifest) -> Option<Repository> {
        cargo.package.repository
            .clone()
            .map(|repo_url| Repository {
                ty: "git".to_string(),
                url: repo_url,
            })
    }

    fn get_js_and_wasm_filenames(cargo: &CargoManifest) -> (String, String) {
        let filename = cargo.package.name.clone().replace("-", "_");
        let js_file = format!("{}.js", filename);
        let wasm_file = format!("{}_bg.wasm", filename);
        (js_file, wasm_file)
    }
}

#[derive(Serialize)]
struct Repository {
    #[serde(rename = "type")]
    ty: String,
    url: String,
}

/// Try to read the `Cargo.toml` file in the given directory.
pub fn read_cargo_toml(path: &str) -> Result<CargoManifest, Error> {
    let manifest_path = format!("{}/Cargo.toml", path);
    let mut cargo_file = File::open(manifest_path)?;
    let mut cargo_contents = String::new();
    cargo_file.read_to_string(&mut cargo_contents)?;

    Ok(toml::from_str(&cargo_contents)?)
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
    let npm_data = NpmPackage::from_manifest(&CARGO_TOML);

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
    Ok(CARGO_TOML.package.name.clone())
}
