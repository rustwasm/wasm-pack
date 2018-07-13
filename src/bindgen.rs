//! Functionality related to installing and running `wasm-bindgen`.

use emoji;
use error::Error;
use progressbar::Step;
use std::process::Command;
use PBAR;

/// Install the `wasm-bindgen` CLI with `cargo install`.
pub fn cargo_install_wasm_bindgen(step: &Step) -> Result<(), Error> {
    let msg = format!("{}Installing WASM-bindgen...", emoji::DOWN_ARROW);
    PBAR.step(step, &msg);
    let output = Command::new("cargo")
        .arg("install")
        .arg("wasm-bindgen-cli")
        .arg("--force")
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        if s.contains("already exists") {
            PBAR.info("wasm-bindgen already installed");
            return Ok(());
        }
        Error::cli("Installing wasm-bindgen failed", s)
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

    let output = Command::new("wasm-bindgen")
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
}
