use console::style;
use emoji;
use failure::Error;
use std::fs;
use std::process::Command;
use PBAR;

pub fn cargo_install_wasm_bindgen() -> Result<(), Error> {
    let step = format!(
        "{} {}Installing WASM-bindgen...",
        style("[6/7]").bold().dim(),
        emoji::DOWN_ARROW
    );
    let pb = PBAR.message(&step);
    let output = Command::new("cargo")
        .arg("install")
        .arg("wasm-bindgen-cli")
        .output()?;
    pb.finish();
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        if s.contains("already exists") {
            PBAR.info("wasm-bindgen already installed");
            return Ok(());
        }
        PBAR.error("Installing wasm-bindgen failed");
        bail!(format!("Details:\n{}", s));
    } else {
        Ok(())
    }
}

pub fn wasm_bindgen_build(path: &str, name: &str) -> Result<(), Error> {
    let step = format!(
        "{} {}Running WASM-bindgen...",
        style("[7/7]").bold().dim(),
        emoji::RUNNER
    );
    let pb = PBAR.message(&step);
    let binary_name = name.replace("-", "_");
    let wasm_path = format!("target/wasm32-unknown-unknown/release/{}.wasm", binary_name);
    let output = Command::new("wasm-bindgen")
        .current_dir(path)
        .arg(&wasm_path)
        .arg("--out-dir")
        .arg("./pkg")
        .output()?;
    pb.finish();
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        PBAR.error("wasm-bindgen failed to execute properly");
        bail!(format!("Details:\n{}", s));
    } else {
        let js_file = format!("{}/pkg/{}.js", path, binary_name);
        let index_file = format!("{}/pkg/index.js", path);
        fs::rename(&js_file, &index_file)?;
        Ok(())
    }
}
