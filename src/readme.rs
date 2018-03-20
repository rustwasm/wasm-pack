use console::style;
use failure::Error;
use std::fs;

use PBAR;
use emoji;

pub fn copy_from_crate(path: &str) -> Result<(), Error> {
    let step = format!(
        "{} {}Copying over your README...",
        style("[5/7]").bold().dim(),
        emoji::DANCERS
    );
    let pb = PBAR.message(&step);
    let crate_readme_path = format!("{}/README.md", path);
    let new_readme_path = format!("{}/pkg/README.md", path);
    if let Err(_) = fs::copy(&crate_readme_path, &new_readme_path) {
        PBAR.warn("origin crate has no README");
    };
    pb.finish();
    Ok(())
}
