use std::fs;
use failure::Error;

pub fn copy_from_crate(path: &str) -> Result<(), Error> {
    let crate_readme_path = format!("{}/README.md", path);
    let new_readme_path = format!("{}/pkg/README.md", path);
    fs::copy(&crate_readme_path, &new_readme_path)?;
    Ok(())
}
