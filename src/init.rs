use std::fs::{copy, create_dir_all, File};
use std::io::prelude::*;
use std::process::Command;

use error::Error;
use manifest::{CargoManifest, NpmPackage};
use serde_json;

// This file contains helper functions for steps to run `wasm-pack init`.
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

/// Step 3: Create a `pkg` directory.
pub fn create_pkg_dir(path: &str) -> Result<(), Error> {
    let pkg_dir_path = format!("{}/pkg", path);
    match create_dir_all(pkg_dir_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::Io(e)),
    }
}

/// Step 4: Write the contents of the `package.json`.
pub fn write_package_json(path: &str, scope: Option<String>, manifest: &CargoManifest) -> Result<(), Error> {
        let pkg_file_path = format!("{}/pkg/package.json", path);
        let mut pkg_file = File::create(pkg_file_path)?;
        let npm_data = NpmPackage::new(manifest, scope);

        let npm_json = serde_json::to_string_pretty(&npm_data)?;
        pkg_file.write_all(npm_json.as_bytes())?;

        Ok(())

        // FIXUP : THIS SHOULD BE REFACTORED.
        // let warn_fmt = |field| {
        //     format!(
        //         "Field {} is missing from Cargo.toml. It is not necessary, but recommended",
        //         field
        //     )
        // };
        // if npm_data.description.is_none() {
        //     self.pbar.warn(&warn_fmt("description"));
        // }
        // if npm_data.repository.is_none() {
        //     self.pbar.warn(&warn_fmt("repository"));
        // }
        // if npm_data.license.is_none() {
        //     self.pbar.warn(&warn_fmt("license"));
        // }
}

// Step 5: Copy the `README` from the crate into the `pkg` directory.
pub fn copy_readme_from_crate(path: &str) -> ::std::io::Result<u64> {
    let crate_readme_path = format!("{}/README.md", path);
    let new_readme_path = format!("{}/pkg/README.md", path);
    copy(&crate_readme_path, &new_readme_path)
}

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

/// Step 7: Run `wasm-bindgen-cli`.
pub fn wasm_bindgen_build(path: &str, name: &str) -> Result<(), Error> {
    let binary_name = name.replace("-", "_");
    let wasm_path = format!("target/wasm32-unknown-unknown/release/{}.wasm", binary_name);
    let output = Command::new("wasm-bindgen")
        .current_dir(path)
        .arg(&wasm_path)
        .arg("--out-dir")
        .arg("./pkg")
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("wasm-bindgen failed to execute properly", s)
    } else {
        Ok(())
    }
}
