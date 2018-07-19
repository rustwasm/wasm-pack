//! Generating `README` files for the packaged wasm.

use error::Error;
use std::fs;
use std::path::Path;

use emoji;
use progressbar::Step;
use PBAR;

/// Copy the crate's README into the `pkg` directory.
pub fn copy_from_crate(path: &Path, step: &Step) -> Result<(), Error> {
    let msg = format!("{}Copying over your README...", emoji::DANCERS);
    PBAR.step(step, &msg);
    let crate_readme_path = path.join("README.md");
    let new_readme_path = path.join("pkg").join("README.md");
    if let Err(_) = fs::copy(&crate_readme_path, &new_readme_path) {
        PBAR.warn("origin crate has no README");
    };
    Ok(())
}
