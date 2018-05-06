use error::Error;
use std::process::Command;

// This file contains the implementations of the steps to run `wasm-pack init`.
// These functions do not interact with the progress bar, and will return
// a result object, representing whether or not the operation completed
// successfully or failed.

/// Step 1: Add the `wasm32-unknown-unknown` target using `rustup`.
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

// Step 2: Compile the crate, targeting `wasm32-unknown-unknown`.
pub fn cargo_build_wasm(path: &str) -> Result<(), Error> {
    let output = Command::new("cargo")
        .current_dir(path)
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("Compilation of your program failed", s)
    } else {
        Ok(())
    }
}

// Step 3: Create a `pkg` directory.
// Step 4: Write the contents of the `package.json`.
// Step 5: Copy the `README` from the crate into the `pkg` directory.

/// Step 6: Install `wasm-bindgen-cli` using `cargo`.
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

// Step 7: Run `wasm-bindgen-cli`.
