use std::fs::File;
use std::io::prelude::*;

use console::style;
use emoji;
use error::Error;
use progressbar::Step;
use serde_json;
use toml;
use PBAR;

#[derive(Deserialize)]
struct CargoManifest {
    package: CargoPackage,
    dependencies: Option<CargoDependencies>,
    lib: Option<CargoLib>,
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

#[derive(Deserialize)]
struct CargoDependencies {
    #[serde(rename = "wasm-bindgen")]
    wasm_bindgen: Option<String>,
}

#[derive(Deserialize)]
struct CargoLib {
    #[serde(rename = "crate-type")]
    crate_type: Option<Vec<String>>,
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
    types: Option<String>,
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
    fn into_npm(mut self, scope: &Option<String>, disable_dts: bool, target: &str) -> NpmPackage {
        let filename = self.package.name.replace("-", "_");
        let wasm_file = format!("{}_bg.wasm", filename);
        let js_file = format!("{}.js", filename);

        let dts_file = if disable_dts == true {
            None
        } else {
            Some(format!("{}.d.ts", filename))
        };

        let js_bg_file = if target == "nodejs" {
            Some(format!("{}_bg.js", filename))
        } else {
            None
        };

        if let Some(s) = scope {
            self.package.name = format!("@{}/{}", s, self.package.name);
        }
        let mut files = vec![wasm_file];

        match dts_file {
            Some(ref dts_file) => {
                files.push(dts_file.to_string());
            }
            None => {}
        }

        match js_bg_file {
            Some(ref js_bg_file) => {
                files.push(js_bg_file.to_string());
            }
            None => {}
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
            files: files,
            main: js_file,
            types: dts_file,
        }
    }
}

/// Generate a package.json file inside in `./pkg`.
pub fn write_package_json(
    path: &str,
    scope: &Option<String>,
    disable_dts: bool,
    target: &str,
    step: &Step,
) -> Result<(), Error> {
    let msg = format!("{}Writing a package.json...", emoji::MEMO);

    let warn_fmt = |field| {
        format!(
            "Field {} is missing from Cargo.toml. It is not necessary, but recommended",
            field
        )
    };

    PBAR.step(step, &msg);
    let pkg_file_path = format!("{}/pkg/package.json", path);
    let mut pkg_file = File::create(pkg_file_path)?;
    let crate_data = read_cargo_toml(path)?;
    let npm_data = crate_data.into_npm(scope, disable_dts, target);

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
    Ok(())
}

pub fn get_crate_name(path: &str) -> Result<String, Error> {
    Ok(read_cargo_toml(path)?.package.name)
}

pub fn check_crate_config(path: &str, step: &Step) -> Result<(), Error> {
    let msg = format!("{}Checking crate configuration...", emoji::WRENCH);
    PBAR.step(&step, &msg);
    check_wasm_bindgen(path)?;
    check_crate_type(path)?;
    Ok(())
}

fn check_wasm_bindgen(path: &str) -> Result<(), Error> {
    if read_cargo_toml(path)?.dependencies.map_or(false, |x| {
        !x.wasm_bindgen.unwrap_or("".to_string()).is_empty()
    }) {
        return Ok(());
    }
    Error::crate_config(&format!(
        "Ensure that you have \"{}\" as a dependency in your Cargo.toml file:\n[dependencies]\nwasm-bindgen = \"0.2\"",
        style("wasm-bindgen").bold().dim()
    ))
}

fn check_crate_type(path: &str) -> Result<(), Error> {
    if read_cargo_toml(path)?.lib.map_or(false, |lib| {
        lib.crate_type
            .map_or(false, |types| types.iter().any(|s| s == "cdylib"))
    }) {
        return Ok(());
    }
    Error::crate_config(
      "crate-type must be cdylib to compile to wasm32-unknown-unknown. Add the following to your Cargo.toml file:\n\n[lib]\ncrate-type = [\"cdylib\"]"
    )
}
