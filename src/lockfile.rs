//! Reading Cargo.lock lock file.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use cargo_metadata;
use error::Error;
use toml;

/// This struct represents the contents of `Cargo.lock`.
#[derive(Deserialize)]
struct Lockfile {
    package: HashMap<String, Package>,
}

/// This struct represents a single package entry in `Cargo.lock`
#[derive(Deserialize)]
struct Package {
    name: String,
    version: String,
    source: Option<String>,
}

/// Get the version of `wasm-bindgen` specified as a dependency.
pub fn get_wasm_bindgen_version(path: &Path) -> Result<Option<String>, Error> {
    let lockfile = read_cargo_lock(&path)?;
    let version = lockfile
        .package
        .get("wasm-bingen")
        .map(|p| p.version.clone());
    Ok(version)
}

/// Read the `Cargo.lock` file for the crate at the given path.
fn read_cargo_lock(crate_path: &Path) -> Result<Lockfile, Error> {
    let lock_path = get_lockfile_path(crate_path)?;
    let mut lockfile = String::new();
    File::open(lock_path)?.read_to_string(&mut lockfile)?;
    toml::from_str(&lockfile).map_err(Error::from)
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
