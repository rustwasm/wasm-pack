//! Utility functions for commands.

use failure;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use walkdir::WalkDir;

/// If an explicit path is given, then use it, otherwise assume the current
/// directory is the crate path.
pub fn set_crate_path(path: Option<PathBuf>) -> Result<PathBuf, failure::Error> {
    Ok(path.unwrap_or_else(|| PathBuf::from(".")))
}

/// Construct our `pkg` directory in the crate.
pub fn create_pkg_dir(out_dir: &Path) -> Result<(), failure::Error> {
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

    WalkDir::new(path)
        .into_iter()
        .filter_map(|x| x.ok().map(|e| e.into_path()))
        .find(|x| is_pkg_directory(&x))
}

fn is_pkg_directory(path: &Path) -> bool {
    path.exists() && path.is_dir() && path.ends_with("pkg")
}

/// Render a `Duration` to a form suitable for display on a console
pub fn elapsed(duration: Duration) -> String {
    let secs = duration.as_secs();

    if secs >= 60 {
        format!("{}m {:02}s", secs / 60, secs % 60)
    } else {
        format!("{}.{:02}s", secs, duration.subsec_nanos() / 10_000_000)
    }
}
