//! Utility functions for commands.

use emoji;
use failure;
use progressbar::Step;
use std::fs;
use std::path::{Path, PathBuf};
use PBAR;

/// If an explicit path is given, then use it, otherwise assume the current
/// directory is the crate path.
pub fn set_crate_path(path: Option<PathBuf>) -> Result<PathBuf, failure::Error> {
    Ok(path.unwrap_or(PathBuf::from(".")))
}

/// Construct our `pkg` directory in the crate.
pub fn create_pkg_dir(out_dir: &Path, step: &Step) -> Result<(), failure::Error> {
    let msg = format!("{}Creating a pkg directory...", emoji::FOLDER);
    PBAR.step(step, &msg);
    fs::create_dir_all(&out_dir)?;
    fs::write(out_dir.join(".gitignore"), "*")?;
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
