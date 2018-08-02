//! Functionality related to checking and running `wasm-opt`.

use emoji;
use error::Error;
use progressbar::Step;
use std::path::Path;
use std::process::Command;
use PBAR;
use manifest::BuildConfig;

/// Check the `wasm-opt` CLI.
fn check_install_wasm_opt(step: &Step) -> Result<(), Error> {
    let msg = format!("{}Checking WASM-opt...", emoji::DOWN_ARROW);
    PBAR.step(step, &msg);
    // check whether `wasm-opt` is installed.
    let output = Command::new("command")
        .args(&["-v", "wasm-opt"])
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        if s.contains("No such file") {
            Error::cli(
                "wasm-opt isn't installed. \
                please follow the build instruction in \
                https://github.com/WebAssembly/binaryen#binaryen",
                s
            )
        }
    } else {
        Ok(())
    }
}

/// Run the `wasm-bindgen` CLI to optimize the current crate's `.wasm`.
pub fn run_wasm_opt(
    path: &Path,
    name: &str,
    build_config: &BuildConfig,
    step: &Step,
) -> Result<(), Error> {
    check_install_wasm_opt(step)?;

    let msg = format!("{}Running WASM-opt...", emoji::RUNNER);
    PBAR.step(step, &msg);
    let binary_name = name.replace("-", "_");
    let release_or_debug = if debug { "debug" } else { "release" };
    let wasm_path = format!(
        "target/wasm32-unknown-unknown/{}/{}.wasm",
        release_or_debug, binary_name
    );

    let output = Command::new("wasm-opt")
        .current_dir(path)
        .arg(&wasm_path)
        // FIXME(csmoe): opt_passes() needs to be supported at upper buildconfig.
        .args(build_config.opt_passes())
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("wasm-opt failed to execute properly", s)
    } else {
        Ok(())
    }
}
