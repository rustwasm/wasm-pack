//! Generating `README` files for the packaged wasm.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::manifest::CrateData;
use crate::PBAR;

/// Copy the crate's README into the `pkg` directory.
pub fn copy_from_crate(crate_data: &CrateData, path: &Path, out_dir: &Path) -> Result<()> {
    assert!(
        fs::metadata(path).ok().map_or(false, |m| m.is_dir()),
        "crate directory should exist"
    );
    assert!(
        fs::metadata(&out_dir).ok().map_or(false, |m| m.is_dir()),
        "crate's pkg directory should exist"
    );

    let crate_readme_path = match crate_data.crate_readme() {
        None => return Ok(()),
        Some(readme_path) => path.join(readme_path),
    };

    let new_readme_path = out_dir.join("README.md");
    if crate_readme_path.exists() {
        fs::copy(&crate_readme_path, &new_readme_path).context("failed to copy README")?;
    } else {
        PBAR.warn("origin crate has no README");
    }
    Ok(())
}
