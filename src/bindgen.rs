//! Functionality related to installing and running `wasm-bindgen`.

use emoji;
use error::Error;
use progressbar::Step;
use std::{env, fs, path, process::Command};
use PBAR;

/// Return a string containing the path to the local `wasm-bindgen`.
fn local_wasm_bindgen_path_str(crate_path: &str) -> String {
    #[cfg(not(target_family = "windows"))]
    return format!("{}/{}", crate_path, "bin/wasm-bindgen");
    #[cfg(target_family = "windows")]
    return format!("{}\\{}", crate_path, "bin\\wasm-bindgen");
}

/// Return a string containing the path to the global `wasm-bindgen`.
fn global_wasm_bindgen_path_str() -> Result<Option<String>, Error> {
    #[cfg(target_family = "windows")]
    let path_sep: &str = ";";
    #[cfg(not(target_family = "windows"))]
    let path_sep: &str = ":";

    let path = env::var("PATH")?;
    for path_dir in path.split(path_sep)
    {
        let prog_str = format!("{}/wasm-bindgen", path_dir);
        if fs::metadata(&prog_str).is_ok() {
            return Ok(Some(prog_str))
        }
    }

    Ok(None)
}

/// Check if the `wasm-bindgen` dependency is locally satisfied.
fn wasm_bindgen_version_check(crate_path: &str, dep_version: &str) -> Result<bool, Error> {
    let path_str = local_wasm_bindgen_path_str(crate_path);
    let wasm_bindgen = path::Path::new(&path_str);

    if !wasm_bindgen.is_file() {
        return Ok(false);
    }

    let output = Command::new(wasm_bindgen).arg("--version").output()?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout);
        let installed_version = s.trim();
        Ok(installed_version == dep_version)
    } else {
        let error_msg = "Could not find version of local wasm-bindgen";
        let s = String::from_utf8_lossy(&output.stderr);
        let e = Error::cli(error_msg, s);
        Err(e)
    }
}

/// Install the `wasm-bindgen` CLI with `cargo install`.
pub fn cargo_install_wasm_bindgen(
    path: &str,
    version: &str,
    install_permitted: bool,
    step: &Step,
) -> Result<(), Error> {
    if wasm_bindgen_version_check(path, version)? {
        let msg = format!("{}WASM-bindgen already installed...", emoji::DOWN_ARROW);
        PBAR.step(step, &msg);
        return Ok(());
    }

    if !install_permitted {
        return Err(Error::crate_config("WASM-bindgen is not installed!"));
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
    debug: bool,
    step: &Step,
) -> Result<(), Error> {
    let msg = format!("{}Running WASM-bindgen...", emoji::RUNNER);
    PBAR.step(step, &msg);
    let binary_name = name.replace("-", "_");
    let release_or_debug = if debug { "debug" } else { "release" };
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

    let wasm_bindgen = local_wasm_bindgen_path_str(path);
    let output = Command::new(wasm_bindgen)
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
}
