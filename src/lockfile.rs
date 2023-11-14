//! Reading Cargo.lock lock file.

#![allow(clippy::new_ret_no_self)]

use std::fs;
use std::path::PathBuf;

use crate::manifest::CrateData;
use anyhow::{anyhow, bail, Context, Result};
use console::style;
use toml;

/// This struct represents the contents of `Cargo.lock`.
#[derive(Clone, Debug, Deserialize)]
pub struct Lockfile {
    package: Vec<Package>,
}

/// This struct represents a single package entry in `Cargo.lock`
#[derive(Clone, Debug, Deserialize)]
struct Package {
    name: String,
    version: String,
}

impl Lockfile {
    /// Read the `Cargo.lock` file for the crate at the given path.
    pub fn new(crate_data: &CrateData) -> Result<Lockfile> {
        let lock_path = get_lockfile_path(crate_data)?;
        let lockfile = fs::read_to_string(&lock_path)
            .with_context(|| anyhow!("failed to read: {}", lock_path.display()))?;
        let lockfile = toml::from_str(&lockfile)
            .with_context(|| anyhow!("failed to parse: {}", lock_path.display()))?;
        Ok(lockfile)
    }

    /// Get the version of `wasm-bindgen` dependency used in the `Cargo.lock`.
    pub fn wasm_bindgen_version(&self) -> Option<&str> {
        self.get_package_version("wasm-bindgen")
    }

    /// Like `wasm_bindgen_version`, except it returns an error instead of
    /// `None`.
    pub fn require_wasm_bindgen(&self) -> Result<&str> {
        self.wasm_bindgen_version().ok_or_else(|| {
            anyhow!(
                "Ensure that you have \"{}\" as a dependency in your Cargo.toml file:\n\
                 [dependencies]\n\
                 wasm-bindgen = \"0.2\"",
                style("wasm-bindgen").bold().dim(),
            )
        })
    }

    /// Get the version of `wasm-bindgen` dependency used in the `Cargo.lock`.
    pub fn wasm_bindgen_test_version(&self) -> Option<&str> {
        self.get_package_version("wasm-bindgen-test")
    }

    fn get_package_version(&self, package: &str) -> Option<&str> {
        self.package
            .iter()
            .find(|p| p.name == package)
            .map(|p| &p.version[..])
    }
}

/// Given the path to the crate that we are building, return a `PathBuf`
/// containing the location of the lock file, by finding the workspace root.
fn get_lockfile_path(crate_data: &CrateData) -> Result<PathBuf> {
    // Check that a lock file can be found in the directory. Return an error
    // if it cannot, otherwise return the path buffer.
    let lockfile_path = crate_data.workspace_root().join("Cargo.lock");
    if !lockfile_path.is_file() {
        bail!("Could not find lockfile at {:?}", lockfile_path)
    } else {
        Ok(lockfile_path)
    }
}
