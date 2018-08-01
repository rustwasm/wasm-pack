//! Functionality related to installing and running `wasm-bindgen`.

use emoji;
use error::Error;
use progressbar::Step;
use std::path::{Path, PathBuf};
use std::process::Command;
use which::which;
use PBAR;

/// Install the `wasm-bindgen` CLI with `cargo install`.
pub fn cargo_install_wasm_bindgen(
    path: &Path,
    version: &str,
    install_permitted: bool,
    step: &Step,
) -> Result<(), Error> {
    // If the `wasm-bindgen` dependency is already met, print a message and return.
    if wasm_bindgen_path(path)
        .map(|bindgen_path| wasm_bindgen_version_check(&bindgen_path, version))
        .unwrap_or(false)
    {
        let msg = format!("{}wasm-bindgen already installed...", emoji::DOWN_ARROW);
        PBAR.step(step, &msg);
        return Ok(());
    }

    // If the `wasm-bindgen` dependency was not met, and installs are not
    // permitted, return a configuration error.
    if !install_permitted {
        let msg = format!("wasm-bindgen v{} is not installed!", version);
        return Error::crate_config(&msg);
    }

    let msg = format!("{}Installing wasm-bindgen...", emoji::DOWN_ARROW);
    PBAR.step(step, &msg);
    let output = Command::new("cargo")
        .arg("install")
        .arg("--force")
        .arg("wasm-bindgen-cli")
        .arg("--version")
        .arg(version)
        .arg("--root")
        .arg(path)
        .output()?;
    if !output.status.success() {
        let message = "Installing wasm-bindgen failed".to_string();
        let s = String::from_utf8_lossy(&output.stderr);
        Err(Error::Cli {
            message,
            stderr: s.to_string(),
        })
    } else {
        Ok(())
    }
}

/// Run the `wasm-bindgen` CLI to generate bindings for the current crate's
/// `.wasm`.
pub fn wasm_bindgen_build(
    path: &Path,
    name: &str,
    disable_dts: bool,
    target: &str,
    debug: bool,
    step: &Step,
) -> Result<(), Error> {
    let msg = format!("{}Running WASM-bindgen...", emoji::RUNNER);
    PBAR.step(step, &msg);
    let binary_name = name.replace("-", "_");
    let release_or_debug = if debug { "debug" } else { "release" };

    if let Some(wasm_bindgen_path) = wasm_bindgen_path(path) {
        let wasm_path = format!(
            "target/wasm32-unknown-unknown/{}/{}.wasm",
            release_or_debug, binary_name
        );
        let dts_arg = if disable_dts == false {
            "--typescript"
        } else {
            "--no-typescript"
        };
        let target_arg = match target {
            "nodejs" => "--nodejs",
            _ => "--browser",
        };
        let bindgen_path = Path::new(&wasm_bindgen_path);
        let output = Command::new(bindgen_path)
            .current_dir(path)
            .arg(&wasm_path)
            .arg("--out-dir")
            .arg("./pkg")
            .arg(dts_arg)
            .arg(target_arg)
            .output()?;
        if !output.status.success() {
            let s = String::from_utf8_lossy(&output.stderr);
            Error::cli("wasm-bindgen failed to execute properly", s)
        } else {
            Ok(())
        }
    } else {
        Error::crate_config("Could not find `wasm-bindgen`")
    }
}

/// Check if the `wasm-bindgen` dependency is locally satisfied.
fn wasm_bindgen_version_check(bindgen_path: &PathBuf, dep_version: &str) -> bool {
    Command::new(bindgen_path)
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .split_whitespace()
                .nth(1)
                .map(|v| v == dep_version)
                .unwrap_or(false)
        }).unwrap_or(false)
}

/// Return a `PathBuf` containing the path to either the local wasm-bindgen
/// version, or the globally installed version if there is no local version.
fn wasm_bindgen_path(crate_path: &Path) -> Option<PathBuf> {
    // Return the path to the local `wasm-bindgen`, if it exists.
    let local_bindgen_path = |crate_path: &Path| -> Option<PathBuf> {
        let mut p = crate_path.to_path_buf();
        p.push("bin");
        p.push("wasm-bindgen");
        if p.is_file() {
            Some(p)
        } else {
            None
        }
    };

    // Return the path to the global `wasm-bindgen`, if it exists.
    let global_bindgen_path = || -> Option<PathBuf> {
        if let Ok(p) = which("wasm-bindgen") {
            Some(p)
        } else {
            None
        }
    };

    local_bindgen_path(crate_path).or_else(global_bindgen_path)
}
