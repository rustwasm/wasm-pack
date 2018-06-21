use failure::{Error, ResultExt};
use std::path::PathBuf;
use std::{fs::File, io::Read};
use toml::value::Table;
use toml::Value;

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

#[derive(Debug, Default, Clone)]
pub(crate) struct BuildConfig {
    pub(crate) manifest_path: Option<PathBuf>,
    pub(crate) build: Option<BuildMode>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct BuildMode {
    pub(crate) debug: Option<WasmPackConfig>,
    pub(crate) profiling: Option<WasmPackConfig>,
    pub(crate) production: Option<WasmPackConfig>,
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

#[derive(Debug, Default, Clone)]
pub(crate) struct WasmPackConfig {
    pub(crate) debug: Option<bool>,
    pub(crate) features: Option<Vec<String>>,
    pub(crate) rustc_opt_level: Option<String>,
    // pub(crate) wasm_opt: Option<WasmOptConfig>,
    // pub(crate) wasm_snip: Option<WasmSnipConfig>,
    pub(crate) wasm_bindgen: Option<bool>,
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

pub(crate) fn parse_config(manifest_path: PathBuf) -> Result<BuildConfig, Error> {
    let cargo_toml: Value = {
        let mut content = String::new();
        File::open(&manifest_path)
            .context("Failed to open Cargo.toml")?
            .read_to_string(&mut content)
            .context("Failed to read Cargo.toml")?;
        content
            .parse::<Value>()
            .context("Failed to parse Cargo.toml")?
    };

    let metadata = cargo_toml
        .get("package")
        .and_then(|table| table.get("metadata"))
        .and_then(|table| table.get("wasm-pack"));
    let metadata = match metadata {
        None => {
            return Ok(BuildConfig {
                manifest_path: Some(manifest_path),
                ..Default::default()
            })
        }
        Some(metadata) => metadata
            .as_table()
            .ok_or_else(|| format_err!("wasm-pack configuration invalid: {:?}", metadata))?,
    };

    let mut config = BuildConfig {
        manifest_path: Some(manifest_path),
        ..Default::default()
    };

    for (key, value) in metadata {
        match (key.as_str(), value.clone()) {
            ("build", Value::Table(t)) => config.build = Some(BuildMode::from_table(t)?),
            // FIXME(csmoe): May be more configs for wasm_pack except `build`
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
