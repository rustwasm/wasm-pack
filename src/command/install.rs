use console::style;
use emoji;
use error::Error;
use std::process::Command;

pub fn rustup_add_wasm_target() -> Result<(), Error> {
    let output = Command::new("rustup")
        .arg("target")
        .arg("add")
        .arg("wasm32-unknown-unknown")
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("Adding the wasm32-unknown-unknown target failed", s)
    } else {
        Ok(())
    }
}

pub fn cargo_install_wasm_bindgen() -> Result<(), Error> {
    let output = Command::new("cargo")
        .arg("install")
        .arg("wasm-bindgen-cli")
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        if s.contains("already exists") {
            // PBAR.info("wasm-bindgen already installed"); // FIXUP:
            return Ok(());
        }
        Error::cli("Installing wasm-bindgen failed", s)
    } else {
        Ok(())
    }
}
