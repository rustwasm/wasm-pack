//! Functionality related to running `wasm-bindgen`.

use binary_install::Download;
use child;
use command::build::{BuildProfile, Target};
use failure::{self, ResultExt};
use install::Tool;
use log::info;
use manifest::CrateData;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Run the `wasm-bindgen` CLI to generate bindings for the current crate's
/// `.wasm`.
pub fn wasm_bindgen_build(
    data: &CrateData,
    bindgen: &Download,
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
    let target_arg = match target {
        Target::Nodejs => "--nodejs",
        Target::NoModules => "--no-modules",
        Target::Web => "--web",
        Target::Bundler => "--browser",
    };
    let bindgen_path = bindgen.binary("wasm-bindgen")?;
    let mut cmd = Command::new(&bindgen_path);
    cmd.arg(&wasm_path)
        .arg("--out-dir")
        .arg(out_dir)
        .arg(dts_arg)
        .arg(target_arg);

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

    let result = child::run(cmd, "wasm-bindgen");
    let result: Result<(), failure::Error> = match result {
        Ok(r) => Ok(r),
        Err(e) => process_error(&bindgen_path, e),
    };
    result.context("Running the wasm-bindgen CLI")?;
    Ok(())
}

fn process_error(bindgen_path: &PathBuf, e: child::CommandError) -> Result<(), failure::Error> {
    match &e.stderr {
        Some(err) if err.trim().starts_with("Unknown flag: '--web'") => {
            let v = wasm_bindgen_get_version(bindgen_path).unwrap_or(String::from("unknown"));
            bail!("Failed to execute `wasm-bindgen`: --web is not supported in version '{}'. Upgrade the wasm-bindgen dependency in Cargo.toml to version 0.2.39 or later.", v)
        }
        Some(err) => {
            eprintln!("{}", err);
            bail!("{}", e.to_string())
        }
        _ => bail!("{}", e.to_string()),
    }
}

/// Check if the `wasm-bindgen` dependency is locally satisfied.
fn wasm_bindgen_version_check(bindgen_path: &PathBuf, dep_version: &str) -> bool {
    wasm_bindgen_get_version(bindgen_path)
        .map(|v| {
            info!(
                "Checking installed `wasm-bindgen` version == expected version: {} == {}",
                v, dep_version
            );
            v == dep_version
        })
        .unwrap_or(false)
}

/// Get the `wasm-bindgen` version
fn wasm_bindgen_get_version(bindgen_path: &PathBuf) -> Option<String> {
    let mut cmd = Command::new(bindgen_path);
    cmd.arg("--version");
    child::run_capture_stdout(cmd, &Tool::WasmBindgen)
        .map(|stdout| match stdout.trim().split_whitespace().nth(1) {
            Some(v) => return Some(v.to_owned()),
            None => return None,
        })
        .unwrap_or(None)
}
