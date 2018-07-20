//! Functionality related to installing and running `wasm-bindgen`.

use emoji;
use error::Error;
use progressbar::Step;
use std::{env, path, process::Command};
use PBAR;

/// Install the `wasm-bindgen` CLI with `cargo install`.
pub fn cargo_install_wasm_bindgen(
    path: &str,
    version: &str,
    install_permitted: bool,
    step: &Step,
) -> Result<(), Error> {
    // Determine if the `wasm-bindgen` dependency is already met for the given mode.
    let dependency_met = if let Some(bindgen_path) = wasm_bindgen_path_str(path, install_permitted)?
    {
        wasm_bindgen_version_check(&bindgen_path, version)?
    } else {
        false
    };

    // If the `wasm-bindgen` dependency is already met, print a message and return.
    if dependency_met {
        let msg = format!("{}WASM-bindgen already installed...", emoji::DOWN_ARROW);
        PBAR.step(step, &msg);
        return Ok(());
    }

    // If the `wasm-bindgen` dependency was not met, and installs are not
    // permitted, return a configuration error.
    if !install_permitted {
        return Err(Error::crate_config("WASM-bindgen is not installed!"));
        // FIXUP: This error message can be improved.
    }

    let msg = format!("{}Installing WASM-bindgen...", emoji::DOWN_ARROW);
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
        let s = String::from_utf8_lossy(&output.stderr);
        Err(Error::cli("Installing wasm-bindgen failed", s))
    } else {
        Ok(())
    }
}

/// Run the `wasm-bindgen` CLI to generate bindings for the current crate's
/// `.wasm`.
pub fn wasm_bindgen_build(
    path: &str,
    name: &str,
    disable_dts: bool,
    target: &str,
    install_permitted: bool,
    debug: bool,
    step: &Step,
) -> Result<(), Error> {
    let msg = format!("{}Running WASM-bindgen...", emoji::RUNNER);
    PBAR.step(step, &msg);
    let binary_name = name.replace("-", "_");
    let release_or_debug = if debug { "debug" } else { "release" };

    if let Some(wasm_bindgen_path_str) = wasm_bindgen_path_str(path, install_permitted)? {
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

        let bindgen_path = path::Path::new(&wasm_bindgen_path_str);
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
            Err(Error::cli("wasm-bindgen failed to execute properly", s))
        } else {
            Ok(())
        }
    } else {
        Err(Error::crate_config("Could not find `wasm-bindgen`"))
    }
}

/// Check if the `wasm-bindgen` dependency is locally satisfied.
fn wasm_bindgen_version_check(bindgen_path: &str, dep_version: &str) -> Result<bool, Error> {
    let wasm_bindgen = path::Path::new(bindgen_path);

    let output = Command::new(wasm_bindgen).arg("--version").output()?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout);
        let installed_version = s.trim();
        Ok(installed_version == dep_version)
    } else {
        // FIXUP: This error message can be improved.
        let error_msg = "Could not find version of local wasm-bindgen";
        let s = String::from_utf8_lossy(&output.stderr);
        let e = Error::cli(error_msg, s);
        Err(e)
    }
}

/// Return a string to `wasm-bindgen`, if it exists for the given mode.
fn wasm_bindgen_path_str(
    crate_path: &str,
    install_permitted: bool,
) -> Result<Option<String>, Error> {
    if install_permitted {
        if let path_str @ Some(_) = local_wasm_bindgen_path_str(crate_path) {
            return Ok(path_str);
        }
    }

    global_wasm_bindgen_path_str()
}

/// Return a string containing the path to the local `wasm-bindgen`, if it exists.
fn local_wasm_bindgen_path_str(crate_path: &str) -> Option<String> {
    #[cfg(not(target_family = "windows"))]
    let local_path = format!("{}/{}", crate_path, "bin/wasm-bindgen");
    #[cfg(target_family = "windows")]
    let local_path = format!("{}\\{}", crate_path, "bin\\wasm-bindgen");

    if path::Path::new(&local_path).is_file() {
        Some(local_path)
    } else {
        None
    }
}

/// Return a string containing the path to the global `wasm-bindgen`, if it exists.
fn global_wasm_bindgen_path_str() -> Result<Option<String>, Error> {
    #[cfg(target_family = "windows")]
    let path_sep: &str = ";";
    #[cfg(not(target_family = "windows"))]
    let path_sep: &str = ":";

    let path = env::var("PATH")?;
    for path_dir in path.split(path_sep) {
        let prog_str = format!("{}/wasm-bindgen", path_dir);
        if path::Path::new(&prog_str).is_file() {
            return Ok(Some(prog_str));
        }
    }

    Ok(None)
}
