use error::Error;
use std::fs;

use emoji;
use progressbar::Step;
use PBAR;

pub fn copy_from_crate(path: &str, step: &Step) -> Result<(), Error> {
    let msg = format!("{}Copying over your README...", emoji::DANCERS);
    PBAR.step(step, &msg)?;
    let crate_readme_path = format!("{}/README.md", path);
    let new_readme_path = format!("{}/pkg/README.md", path);
    if let Err(_) = fs::copy(&crate_readme_path, &new_readme_path) {
        PBAR.warn("origin crate has no README")?;
    };
    Ok(())
}
