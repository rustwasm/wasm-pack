//! Functionality related to running `wasm-bindgen`.

use child;
use command::build::{BuildProfile, Target};
use failure::{self, ResultExt};
use install::{self, Tool};
use manifest::CrateData;
use semver;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Run the `wasm-bindgen` CLI to generate bindings for the current crate's
/// `.wasm`.
pub fn wasm_bindgen_build(
    data: &CrateData,
    install_status: &install::Status,
    out_dir: &Path,
    out_name: &Option<String>,
    disable_dts: bool,
    target: Target,
    profile: BuildProfile,
) -> Result<(), failure::Error> {
    let release_or_debug = match profile {
        BuildProfile::Release | BuildProfile::Profiling => "release",
        BuildProfile::Dev => "debug",
    };

    let out_dir = out_dir.to_str().unwrap();

    let wasm_path = data
        .target_directory()
        .join("wasm32-unknown-unknown")
        .join(release_or_debug)
        .join(data.crate_name())
        .with_extension("wasm");

    let dts_arg = if disable_dts {
        "--no-typescript"
    } else {
        "--typescript"
    };
    let bindgen_path = install::get_tool_path(install_status, Tool::WasmBindgen)?
        .binary(&Tool::WasmBindgen.to_string())?;

    let mut cmd = Command::new(&bindgen_path);
    cmd.arg(&wasm_path)
        .arg("--out-dir")
        .arg(out_dir)
        .arg(dts_arg);

    let target_arg = build_target_arg(target, &bindgen_path)?;
    if supports_dash_dash_target(bindgen_path.to_path_buf())? {
        cmd.arg("--target").arg(target_arg);
    } else {
        cmd.arg(target_arg);
    }

    if let Some(value) = out_name {
        cmd.arg("--out-name").arg(value);
    }

    let profile = data.configured_profile(profile);
    if profile.wasm_bindgen_debug_js_glue() {
        cmd.arg("--debug");
    }
    if !profile.wasm_bindgen_demangle_name_section() {
        cmd.arg("--no-demangle");
    }
    if profile.wasm_bindgen_dwarf_debug_info() {
        cmd.arg("--keep-debug");
    }

    child::run(cmd, "wasm-bindgen").context("Running the wasm-bindgen CLI")?;
    Ok(())
}

/// Run the `wasm2es6js` CLI to inline .wasm as base64 string into .js file
pub fn wasm_inline_base64(
    data: &CrateData,
    install_status: &install::Status,
    out_dir: &Path,
    out_name: &Option<String>,
    disable_dts: bool,
    target: Target,
) -> Result<(), failure::Error> {
    let name_prefix = match out_name {
        Some(value) => value.clone(),
        None => data.crate_name(),
    };

    if target == Target::Bundler {
        let wasm_file = out_dir.join(format!("{}_bg.wasm", name_prefix));
        let wasm2js_path =
            install::get_tool_path(install_status, Tool::WasmBindgen)?.binary("wasm2es6js")?;
        let mut cmd = Command::new(&wasm2js_path);
        cmd.arg(&wasm_file)
            .arg("--base64")
            .arg("--out-dir")
            .arg(out_dir);
        if !disable_dts {
            cmd.arg("--typescript");
        }

        child::run(cmd, "wasm2es6js").context("Running the wasm2es6js CLI")?;
    }

    // Remove wasm import from entry js file
    // and replace wasm loading logic
    let entry_js_file_path = out_dir.join(format!("{}.js", name_prefix));
    if entry_js_file_path.exists() {
        let mut base64_encoded_wasm: Option<String> = None;
        let content = fs::read_to_string(&entry_js_file_path)?;
        let mut writer = fs::File::create(entry_js_file_path)?;
        for line in content.split("\n") {
            if line.contains("import * as wasm from") {
                continue;
            } else if line.contains("const bytes = ") {
                if base64_encoded_wasm.is_none() {
                    let wasm_file_path = out_dir.join(format!("{}_bg.wasm", name_prefix));
                    if wasm_file_path.exists() {
                        let wasm_bytes = fs::read(wasm_file_path)?;
                        base64_encoded_wasm = Some(base64::encode(&wasm_bytes));
                    }
                }
                if let Some(s) = &base64_encoded_wasm {
                    writeln!(&mut writer, "const wasmBase64 = '{}'", s)?;
                    writeln!(&mut writer, "const bytes = (typeof Buffer === 'undefined') ? Uint8Array.from(atob(wasmBase64), c => c.charCodeAt(0)) : Buffer.from(wasmBase64, 'base64')")?;
                }
            } else {
                writeln!(&mut writer, "{}", line.trim())?;
            }
        }
    }

    Ok(())
}

/// Check if the `wasm-bindgen` dependency is locally satisfied for the web target
fn supports_web_target(cli_path: &PathBuf) -> Result<bool, failure::Error> {
    let cli_version = semver::Version::parse(&install::get_cli_version(
        &install::Tool::WasmBindgen,
        cli_path,
    )?)?;
    let expected_version = semver::Version::parse("0.2.39")?;
    Ok(cli_version >= expected_version)
}

/// Check if the `wasm-bindgen` dependency is locally satisfied for the --target flag
fn supports_dash_dash_target(cli_path: PathBuf) -> Result<bool, failure::Error> {
    let cli_version = semver::Version::parse(&install::get_cli_version(
        &install::Tool::WasmBindgen,
        &cli_path,
    )?)?;
    let expected_version = semver::Version::parse("0.2.40")?;
    Ok(cli_version >= expected_version)
}

fn build_target_arg(target: Target, cli_path: &PathBuf) -> Result<String, failure::Error> {
    if !supports_dash_dash_target(cli_path.to_path_buf())? {
        Ok(build_target_arg_legacy(target, cli_path)?)
    } else {
        Ok(target.to_string())
    }
}

fn build_target_arg_legacy(target: Target, cli_path: &PathBuf) -> Result<String, failure::Error> {
    log::info!("Your version of wasm-bindgen is out of date. You should consider updating your Cargo.toml to a version >= 0.2.40.");
    let target_arg = match target {
        Target::Nodejs => "--nodejs",
        Target::NoModules => "--no-modules",
        Target::Web => {
            if supports_web_target(&cli_path)? {
                "--web"
            } else {
                bail!("Your current version of wasm-bindgen does not support the 'web' target. Please update your project to wasm-bindgen version >= 0.2.39.")
            }
        }
        Target::Bundler => "--browser",
    };
    Ok(target_arg.to_string())
}
