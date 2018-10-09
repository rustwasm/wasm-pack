//! Reading and writing Cargo.toml and package.json manifests.

mod npm;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use self::npm::{
    repository::Repository, CommonJSPackage, ESModulesPackage, NoModulesPackage, NpmPackage,
};
use emoji;
use error::Error;
use failure;
use progressbar::Step;
use serde_json;
use toml;
use PBAR;

#[derive(Debug, Deserialize)]
struct CargoManifest {
    package: CargoPackage,
    dependencies: Option<HashMap<String, CargoDependency>>,
    #[serde(rename = "dev-dependencies")]
    dev_dependencies: Option<HashMap<String, CargoDependency>>,
    lib: Option<CargoLib>,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    name: String,
    authors: Vec<String>,
    description: Option<String>,
    version: String,
    license: Option<String>,
    repository: Option<String>,
}

impl CargoPackage {
    fn check_optional_fields(&self) {
        let mut messages = vec![];
        if self.description.is_none() {
            messages.push("description");
        }
        if self.repository.is_none() {
            messages.push("repository");
        }
        if self.license.is_none() {
            messages.push("license");
        }

        match messages.len() {
            1 => PBAR.info(&format!("Optional field missing from Cargo.toml: '{}'. This is not necessary, but recommended", messages[0])),
            2 => PBAR.info(&format!("Optional fields missing from Cargo.toml: '{}', '{}'. These are not necessary, but recommended", messages[0], messages[1])),
            3 => PBAR.info(&format!("Optional fields missing from Cargo.toml: '{}', '{}', and '{}'. These are not necessary, but recommended", messages[0], messages[1], messages[2])),
            _ => ()
        };
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CargoDependency {
    Simple(String),
    Detailed(DetailedCargoDependency),
}

#[derive(Debug, Deserialize)]
struct DetailedCargoDependency {
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoLib {
    #[serde(rename = "crate-type")]
    crate_type: Option<Vec<String>>,
}

fn read_cargo_toml(path: &Path) -> Result<CargoManifest, failure::Error> {
    let manifest_path = path.join("Cargo.toml");
    if !manifest_path.is_file() {
        return Err(Error::crate_config(&format!(
            "Crate directory is missing a `Cargo.toml` file; is `{}` the wrong directory?",
            path.display()
        ))
        .into());
    }
    let mut cargo_file = File::open(manifest_path)?;
    let mut cargo_contents = String::new();
    cargo_file.read_to_string(&mut cargo_contents)?;

    let manifest: CargoManifest = toml::from_str(&cargo_contents)?;
    Ok(manifest)
}

impl CargoManifest {
    fn into_commonjs(mut self, scope: &Option<String>, disable_dts: bool) -> NpmPackage {
        let filename = self.package.name.replace("-", "_");
        let wasm_file = format!("{}_bg.wasm", filename);
        let js_file = format!("{}.js", filename);
        let mut files = vec![wasm_file];

        let js_bg_file = format!("{}_bg.js", filename);
        files.push(js_bg_file.to_string());

        if let Some(s) = scope {
            self.package.name = format!("@{}/{}", s, self.package.name);
        }

        let dts_file = if disable_dts == false {
            let file = format!("{}.d.ts", filename);
            files.push(file.to_string());
            Some(file)
        } else {
            None
        };

        &self.package.check_optional_fields();

        NpmPackage::CommonJSPackage(CommonJSPackage {
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
        })
    }

    fn into_esmodules(mut self, scope: &Option<String>, disable_dts: bool) -> NpmPackage {
        let filename = self.package.name.replace("-", "_");
        let wasm_file = format!("{}_bg.wasm", filename);
        let js_file = format!("{}.js", filename);
        let mut files = vec![wasm_file, js_file.clone()];

        let dts_file = if disable_dts == false {
            let file = format!("{}.d.ts", filename);
            files.push(file.to_string());
            Some(file)
        } else {
            None
        };

        if let Some(s) = scope {
            self.package.name = format!("@{}/{}", s, self.package.name);
        }

        &self.package.check_optional_fields();

        NpmPackage::ESModulesPackage(ESModulesPackage {
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
            module: js_file,
            types: dts_file,
            side_effects: "false".to_string(),
        })
    }

    fn into_nomodules(mut self, scope: &Option<String>, disable_dts: bool) -> NpmPackage {
        let filename = self.package.name.replace("-", "_");
        let wasm_file = format!("{}_bg.wasm", filename);
        let js_file = format!("{}.js", filename);
        let mut files = vec![wasm_file, js_file.clone()];

        let dts_file = if disable_dts == false {
            let file = format!("{}.d.ts", filename);
            files.push(file.to_string());
            Some(file)
        } else {
            None
        };

        if let Some(s) = scope {
            self.package.name = format!("@{}/{}", s, self.package.name);
        }

        &self.package.check_optional_fields();

        NpmPackage::NoModulesPackage(NoModulesPackage {
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
            browser: js_file,
            types: dts_file,
        })
    }
}

/// Generate a package.json file inside in `./pkg`.
pub fn write_package_json(
    path: &Path,
    out_dir: &Path,
    scope: &Option<String>,
    disable_dts: bool,
    target: &str,
    step: &Step,
) -> Result<(), failure::Error> {
    let msg = format!("{}Writing a package.json...", emoji::MEMO);

    PBAR.step(step, &msg);
    let pkg_file_path = out_dir.join("package.json");
    let mut pkg_file = File::create(pkg_file_path)?;
    let crate_data = read_cargo_toml(path)?;
    let npm_data = if target == "nodejs" {
        crate_data.into_commonjs(scope, disable_dts)
    } else if target == "no-modules" {
        crate_data.into_nomodules(scope, disable_dts)
    } else {
        crate_data.into_esmodules(scope, disable_dts)
    };

    let npm_json = serde_json::to_string_pretty(&npm_data)?;
    pkg_file.write_all(npm_json.as_bytes())?;
    Ok(())
}

/// Get the crate name for the crate at the given path.
pub fn get_crate_name(path: &Path) -> Result<String, failure::Error> {
    Ok(read_cargo_toml(path)?.package.name)
}

/// Check that the crate the given path is properly configured.
pub fn check_crate_config(path: &Path, step: &Step) -> Result<(), failure::Error> {
    let msg = format!("{}Checking crate configuration...", emoji::WRENCH);
    PBAR.step(&step, &msg);
    check_crate_type(path)?;
    Ok(())
}

fn check_crate_type(path: &Path) -> Result<(), failure::Error> {
    if read_cargo_toml(path)?.lib.map_or(false, |lib| {
        lib.crate_type
            .map_or(false, |types| types.iter().any(|s| s == "cdylib"))
    }) {
        return Ok(());
    }
    Err(Error::crate_config(
      "crate-type must be cdylib to compile to wasm32-unknown-unknown. Add the following to your \
       Cargo.toml file:\n\n\
       [lib]\n\
       crate-type = [\"cdylib\", \"rlib\"]"
    ).into())
}
