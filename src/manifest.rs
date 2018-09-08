//! Reading and writing Cargo.toml and package.json manifests.

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use console::style;
use emoji;
use error::Error;
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

fn normalize_dependency_name(dep: &str) -> String {
    dep.replace("-", "_")
}

fn normalize_dependencies(
    deps: HashMap<String, CargoDependency>,
) -> HashMap<String, CargoDependency> {
    let mut new_deps = HashMap::with_capacity(deps.len());
    for (key, val) in deps {
        new_deps.insert(normalize_dependency_name(&key), val);
    }
    new_deps
}

impl CargoManifest {
    fn normalize_dependencies(&mut self) {
        if let Some(deps) = self.dependencies.take() {
            self.dependencies = Some(normalize_dependencies(deps));
        }
        if let Some(dev_deps) = self.dev_dependencies.take() {
            self.dev_dependencies = Some(normalize_dependencies(dev_deps));
        }
    }
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

#[derive(Serialize)]
struct NpmPackage {
    name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    collaborators: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repository: Option<Repository>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    files: Vec<String>,
    main: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    types: Option<String>,
}

#[derive(Serialize)]
struct Repository {
    #[serde(rename = "type")]
    ty: String,
    url: String,
}

fn read_cargo_toml(path: &Path) -> Result<CargoManifest, Error> {
    let manifest_path = path.join("Cargo.toml");
    if !manifest_path.is_file() {
        return Error::crate_config(&format!(
            "Crate directory is missing a `Cargo.toml` file; is `{}` the wrong directory?",
            path.display()
        )).map(|_| unreachable!());
    }
    let mut cargo_file = File::open(manifest_path)?;
    let mut cargo_contents = String::new();
    cargo_file.read_to_string(&mut cargo_contents)?;

    let mut manifest: CargoManifest = toml::from_str(&cargo_contents)?;
    manifest.normalize_dependencies();

    Ok(manifest)
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
    path: &Path,
    out_dir: &Path,
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
    let pkg_file_path = out_dir.join("package.json");
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

/// Get the crate name for the crate at the given path.
pub fn get_crate_name(path: &Path) -> Result<String, Error> {
    Ok(read_cargo_toml(path)?.package.name)
}

/// Check that the crate the given path is properly configured.
pub fn check_crate_config(path: &Path, step: &Step) -> Result<(), Error> {
    let msg = format!("{}Checking crate configuration...", emoji::WRENCH);
    PBAR.step(&step, &msg);
    check_wasm_bindgen(path)?;
    check_wasm_bindgen_test(path)?;
    check_crate_type(path)?;
    Ok(())
}

fn check_wasm_bindgen(path: &Path) -> Result<(), Error> {
    get_wasm_bindgen_version(path)?;
    Ok(())
}

fn check_wasm_bindgen_test(path: &Path) -> Result<(), Error> {
    let expected_version = get_wasm_bindgen_version(path)?;

    // Only do the version check if `wasm-bindgen-test` is actually a
    // dependency. Not every crate needs to have tests!
    if let Ok(actual_version) = get_wasm_bindgen_test_version(path) {
        if expected_version != actual_version {
            return Error::crate_config(&format!(
                "The `wasm-bindgen-test` dependency version ({}) must match \
                 the `wasm-bindgen` dependency version ({}), but it does not.",
                actual_version, expected_version
            ));
        }
    }

    Ok(())
}

fn check_crate_type(path: &Path) -> Result<(), Error> {
    if read_cargo_toml(path)?.lib.map_or(false, |lib| {
        lib.crate_type
            .map_or(false, |types| types.iter().any(|s| s == "cdylib"))
    }) {
        return Ok(());
    }
    Error::crate_config(
      "crate-type must be cdylib to compile to wasm32-unknown-unknown. Add the following to your \
       Cargo.toml file:\n\n\
       [lib]\n\
       crate-type = [\"cdylib\"]"
    )
}

fn get_dependency_version(
    dependencies: Option<&HashMap<String, CargoDependency>>,
    dependency: &str,
    dependencies_section_name: &str,
    version_suggestion: &str,
) -> Result<String, Error> {
    if let Some(deps) = dependencies {
        let dependency = normalize_dependency_name(dependency);
        match deps.get(&dependency) {
            Some(CargoDependency::Simple(version))
            | Some(CargoDependency::Detailed(DetailedCargoDependency {
                version: Some(version),
            })) => Ok(version.clone()),
            Some(CargoDependency::Detailed(DetailedCargoDependency { version: None })) => {
                let msg = format!(
                    "\"{}\" dependency is missing its version number",
                    style(&dependency).bold().dim()
                );
                Err(Error::CrateConfig { message: msg })
            }
            None => {
                let message = format!(
                    "Ensure that you have \"{}\" as a dependency in your Cargo.toml file:\n\
                     [{}]\n\
                     {} = \"{}\"",
                    style(&dependency).bold().dim(),
                    dependencies_section_name,
                    dependency,
                    version_suggestion
                );
                Err(Error::CrateConfig { message })
            }
        }
    } else {
        let message = String::from("Could not find crate dependencies");
        Err(Error::CrateConfig { message })
    }
}

/// Get the version of `wasm-bindgen` specified as a dependency.
pub fn get_wasm_bindgen_version(path: &Path) -> Result<String, Error> {
    let toml = read_cargo_toml(path)?;
    get_dependency_version(
        toml.dependencies.as_ref(),
        "wasm-bindgen",
        "dependencies",
        "0.2",
    )
}

/// Get the version of `wasm-bindgen-test` specified as a dependency.
pub fn get_wasm_bindgen_test_version(path: &Path) -> Result<String, Error> {
    let toml = read_cargo_toml(path)?;
    get_dependency_version(
        toml.dev_dependencies.as_ref(),
        "wasm-bindgen-test",
        "dev-dependencies",
        "0.2",
    )
}
