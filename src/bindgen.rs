use console::style;
use emoji;
use error::Error;
use std::{env, fs, process::Command};
use PBAR;

#[cfg(target_family = "windows")]
static PATH_SEP: &str = ";";

#[cfg(not(target_family = "windows"))]
static PATH_SEP: &str = ":";

pub fn cargo_install_wasm_bindgen() -> Result<(), Error> {
    if wasm_bindgen_installed()? {
        return Ok(());
    }
    let step = format!(
        "{} {}Installing WASM-bindgen...",
        style("[6/7]").bold().dim(),
        emoji::DOWN_ARROW
    );
    let pb = PBAR.message(&step);
    let output = Command::new("cargo")
        .arg("install")
        .arg("wasm-bindgen-cli")
        .arg("--force")
        .output()?;
    pb.finish();
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

pub fn wasm_bindgen_build(
    path: &str,
    name: &str,
    disable_dts: bool,
    target: String,
) -> Result<(), Error> {
    let step = format!(
        "{} {}Running WASM-bindgen...",
        style("[7/7]").bold().dim(),
        emoji::RUNNER
    );
    let pb = PBAR.message(&step);
    let binary_name = name.replace("-", "_");
    let wasm_path = format!("target/wasm32-unknown-unknown/release/{}.wasm", binary_name);

    let dts_arg = if disable_dts == false {
        "--typescript"
    } else {
        "--no-typescript"
    };

    let target_arg = match target.as_str() {
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
    pb.finish();
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("wasm-bindgen failed to execute properly", s)
    } else {
        Ok(())
    }
}

fn wasm_bindgen_installed() -> Result<bool, Error> {
    let path = env::var("PATH")?;
    let is_installed = path.split(PATH_SEP)
        .map(|p: &str| -> bool {
            let prog_str = format!("{}/wasm-bindgen", p);
            fs::metadata(prog_str).is_ok()
        })
        .fold(false, |res, b| res || b);
    Ok(is_installed)
}
