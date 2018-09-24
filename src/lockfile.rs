//! Reading Cargo.lock lock file.

use std::fs;
use std::path::{Path, PathBuf};

use cargo_metadata;
use console::style;
use error::Error;
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
    pub fn new(crate_path: &Path) -> Result<Lockfile, Error> {
        let lock_path = get_lockfile_path(crate_path)?;
        let lockfile = fs::read_to_string(lock_path)?;
        toml::from_str(&lockfile).map_err(Error::from)
    }

    /// Get the version of `wasm-bindgen` dependency used in the `Cargo.lock`.
    pub fn wasm_bindgen_version(&self) -> Option<&str> {
        self.get_package_version("wasm-bindgen")
    }

    /// Like `wasm_bindgen_version`, except it returns an error instead of
    /// `None`.
    pub fn require_wasm_bindgen(&self) -> Result<&str, Error> {
        self.wasm_bindgen_version().ok_or_else(|| {
            let message = format!(
                "Ensure that you have \"{}\" as a dependency in your Cargo.toml file:\n\
                 [dependencies]\n\
                 wasm-bindgen = \"0.2\"",
                style("wasm-bindgen").bold().dim(),
            );
            Error::CrateConfig { message }
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

/// Given the path to the crate that we are buliding, return a `PathBuf`
/// containing the location of the lock file, by finding the workspace root.
fn get_lockfile_path(crate_path: &Path) -> Result<PathBuf, Error> {
    // Identify the crate's root directory, or return an error.
    let manifest = crate_path.join("Cargo.toml");
    let crate_root = cargo_metadata::metadata(Some(&manifest))
        .map_err(|_| Error::CrateConfig {
            message: String::from("Error while processing crate metadata"),
        })?.workspace_root;
    // Check that a lock file can be found in the directory. Return an error
    // if it cannot, otherwise return the path buffer.
    let lockfile_path = Path::new(&crate_root).join("Cargo.lock");
    if !lockfile_path.is_file() {
        Err(Error::CrateConfig {
            message: format!("Could not find lockfile at {:?}", lockfile_path),
        })
    } else {
        Ok(lockfile_path)
    }
}
