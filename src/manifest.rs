//! Reading and writing Cargo.toml and package.json manifests.

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use console::style;
use emoji;
use error::Error;
use progressbar::Step;
use serde_json;
use toml::{self, value::Table, Value};
use PBAR;

/// Manifest in `Cargo.toml`.
#[derive(Deserialize)]
pub struct CargoManifest {
    package: CargoPackage,
    dependencies: Option<HashMap<String, CargoDependency>>,
    lib: Option<CargoLib>,
    /// build config.
    pub build_config: Option<BuildConfig>,
}

/// Build configuration.
#[derive(Deserialize, Debug, Default, Clone)]
pub struct BuildConfig {
    build: Option<BuildMode>,
}

/// Build mode.
#[derive(Deserialize, Debug, Default, Clone)]
pub struct BuildMode {
    debug: Option<WasmPackConfig>,
    profiling: Option<WasmPackConfig>,
    production: Option<WasmPackConfig>,
}

// TODO(csmoe): introduce wasm-* config
// use xxx::snip::SnipOptions as WasmSnipConfig;

// macro_rules! tool_config {
//     ($wasm_tool: ident,$config: ident) => {
//         #[derive(Debug, Clone)]
//         enum $wasm_tool {
//             Bool(bool),
//             Config($config),
//         }
//
//         impl Default for $wasm_tool {
//             fn default() -> Self {
//                 $wasm_tool::Bool(false)
//             }
//         }
//     };
// }
//
// tool_config!(WasmOpt, WasmOptConfig);
// tool_config!(WasmSnip, WasmSnipConfig);

/// wasm-pack config.
#[derive(Deserialize, Debug, Default, Clone)]
pub struct WasmPackConfig {
    debug: Option<bool>,
    features: Option<Vec<String>>,
    rustc_opt_level: Option<String>,
    // pub wasm_opt: Option<WasmOptConfig>,
    // pub wasm_snip: Option<WasmSnipConfig>,
    wasm_bindgen: Option<bool>,
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
#[serde(untagged)]
enum CargoDependency {
    Simple(String),
    Detailed(DetailedCargoDependency),
}

#[derive(Deserialize)]
struct DetailedCargoDependency {}

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

/// Read cargo toml.
pub fn read_cargo_toml(path: &PathBuf) -> Result<CargoManifest, Error> {
    let manifest_path = path.join("Cargo.toml");
    let mut cargo_file = File::open(manifest_path)?;
    let mut cargo_contents = String::new();
    cargo_file.read_to_string(&mut cargo_contents)?;

    let build_toml = cargo_contents.parse::<toml::Value>()?;
    let build_config = parse_build_config(&build_toml);

    let mut cargo_manifest: CargoManifest = toml::from_str(&cargo_contents)?;
    cargo_manifest.build_config = build_config.ok();

    Ok(cargo_manifest)
}

fn parse_build_config(cargo_toml: &toml::Value) -> Result<BuildConfig, Error> {
    let metadata = cargo_toml
        .get("package")
        .and_then(|table| table.get("metadata"))
        .and_then(|table| table.get("wasm-pack"));
    let metadata = match metadata {
        None => return Ok(BuildConfig::default()),
        Some(metadata) => metadata
            .as_table()
            .ok_or_else(|| format_err!("wasm-pack configuration invalid: {:?}", metadata))?,
    };

    let mut config = BuildConfig::default();
    for (key, value) in metadata {
        match (key.as_str(), value.clone()) {
            ("build", Value::Table(t)) => config.build = Some(BuildMode::from_table(t)?),
            (key, value) => Err(format_err!(
                "unexpected \
                 `package.metadata.wasm_pack` key `{}` with value `{}`",
                key,
                value
            ))?,
        }
    }

    Ok(config)
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

impl BuildMode {
    fn from_table(table: Table) -> Result<BuildMode, Error> {
        let mut build_mode = BuildMode::default();
        for (key, value) in table {
            match (key.as_str(), value.clone()) {
                ("debug", Value::Table(t)) => {
                    build_mode.debug = Some(WasmPackConfig::from_table(t)?)
                }
                ("profiling", Value::Table(t)) => {
                    build_mode.profiling = Some(WasmPackConfig::from_table(t)?);
                }
                ("production", Value::Table(t)) => {
                    build_mode.production = Some(WasmPackConfig::from_table(t)?);
                }
                (key, value) => Err(format_err!(
                    "unexpected \
                     `package.metadata.wasm_pack.build` key `{}` with value `{}`",
                    key,
                    value
                ))?,
            }
        }
        Ok(build_mode)
    }
}

impl WasmPackConfig {
    fn from_table(table: Table) -> Result<WasmPackConfig, Error> {
        let mut wasm_pack_config = WasmPackConfig::default();
        for (key, value) in table {
            match (key.as_str(), value.clone()) {
                ("debug", Value::Boolean(b)) => wasm_pack_config.debug = From::from(b),
                ("features", Value::Array(a)) => {
                    let mut features = Vec::new();
                    for value in a {
                        match value {
                            Value::String(s) => features.push(s),
                            _ => Err(format_err!("features must be a list of strings"))?,
                        }
                    }
                    wasm_pack_config.features = Some(features);
                }
                ("rustc-opt-level", Value::String(s)) => {
                    wasm_pack_config.rustc_opt_level = From::from(s)
                }
                // ("wasm-opt", opt) => match opt {
                //     WasmOpt::Bool(Value::Boolean(b)) => wasm_pack_config.wasm_opt = From::from(b),
                //     WasmOpt::Config(Value::Table(t)) => {
                //         let mut opt_config = WasmOptConfig::default();
                //         for (key, value) in t {
                //             match (key.as_str(), value.clone()) {
                //                 ("opt-level", Value::String(s)) => {
                //                     opt_config.opt_level = From::from(s)
                //                 }
                //             }
                //         }
                //         wasm_pack_config.wasm_opt = Some(opt_config);
                //     }
                // },
                // ("wasm-snip", snip) => match snip {
                //     WasmSnip::Bool(Value::Boolean(b)) => wasm_pack_config.wasm_snip = From::from(b),
                //     WasmSnip::Config(Value::Table(t)) => {
                //         let mut snip_config = WasmSnipConfig::default();
                //         for (key, value) in t {
                //             match (key.as_str(), value.clone()) {
                //                 ("snip-rust-fmt-code", Value::Boolean(b)) => {
                //                     snip_config.snip_rust_fmt_code = From::from(b)
                //                 }
                //                 ("snip-rust-panicking-code", Value::Boolean(b)) => {
                //                     snip_config.snip_rust_panicking_code = From::from(b)
                //                 }
                //             }
                //         }
                //         wasm_pack_config.wasm_snip = Some(snip_config);
                //     }
                // },
                ("wasm-bindgen", Value::Boolean(b)) => {
                    wasm_pack_config.wasm_bindgen = From::from(b)
                }
                (key, value) => Err(format_err!(
                    "unexpected \
                     `package.metadata.wasm_pack.build.<buildmode>` key `{}` with value `{}`",
                    key,
                    value
                ))?,
            }
        }
        Ok(wasm_pack_config)
    }
}

/// Generate a package.json file inside in `./pkg`.
pub fn write_package_json(
    path: &PathBuf,
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
    let pkg_file_path = path.join("pkg").join("package.json");
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
pub fn get_crate_name(path: &PathBuf) -> Result<String, Error> {
    Ok(read_cargo_toml(path)?.package.name)
}

/// Check that the crate the given path is properly configured.
pub fn check_crate_config(path: &PathBuf, step: &Step) -> Result<(), Error> {
    let msg = format!("{}Checking crate configuration...", emoji::WRENCH);
    PBAR.step(&step, &msg);
    check_wasm_bindgen(path)?;
    check_crate_type(path)?;
    Ok(())
}

fn check_wasm_bindgen(path: &PathBuf) -> Result<(), Error> {
    let cargo_toml = read_cargo_toml(path)?;
    if cargo_toml
        .dependencies
        .map_or(false, |deps| deps.contains_key("wasm-bindgen"))
    {
        return Ok(());
    }
    Error::crate_config(&format!(
        "Ensure that you have \"{}\" as a dependency in your Cargo.toml file:\n[dependencies]\nwasm-bindgen = \"0.2\"",
        style("wasm-bindgen").bold().dim()
    ))
}

fn check_crate_type(path: &PathBuf) -> Result<(), Error> {
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
