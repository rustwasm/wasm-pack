//! Utility functions for commands.

use emoji;
use error::Error;
use progressbar::Step;
use std::fs;
use std::path::{Path, PathBuf};
use PBAR;

/// If an explicit path is given, then use it, otherwise assume the current
/// directory is the crate path.
pub fn set_crate_path(path: Option<PathBuf>) -> PathBuf {
    let crate_path = match path {
        Some(p) => p,
        None => PathBuf::from("."),
    };

    crate_path
}

/// Construct our `pkg` directory in the crate.
pub fn create_pkg_dir(path: &Path, step: &Step) -> Result<(), Error> {
    let msg = format!("{}Creating a pkg directory...", emoji::FOLDER);
    PBAR.step(step, &msg);
    let pkg_dir_path = path.join("pkg");
    fs::create_dir_all(pkg_dir_path)?;
    Ok(())
}

/// Locates the pkg directory from a specific path
/// Returns None if unable to find the 'pkg' directory
pub fn find_pkg_directory(path: &Path) -> Option<PathBuf> {
    if is_pkg_directory(path) {
        return Some(path.to_owned());
    }

    path.read_dir().ok().and_then(|entries| {
        entries
            .filter_map(|x| x.ok().map(|v| v.path()))
            .find(|x| is_pkg_directory(&x))
    })
}

fn is_pkg_directory(path: &Path) -> bool {
    path.exists() && path.is_dir() && path.ends_with("pkg")
}
