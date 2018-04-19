use PBAR;
use failure::Error;
use std::process::Command;

pub fn npm_pack(path: &str) -> Result<(), Error> {
    let pkg_file_path = format!("{}/pkg", path);
    let output = Command::new("npm")
        .current_dir(pkg_file_path)
        .arg("pack")
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        PBAR.error("Packaging up your code failed");
        bail!(format!("Details:\n{}", s));
    } else {
        Ok(())
    }
}

pub fn npm_publish(path: &str) -> Result<(), Error> {
    let pkg_file_path = format!("{}/pkg", path);
    let output = Command::new("npm")
        .current_dir(pkg_file_path)
        .arg("publish")
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        PBAR.error("Publishing to npm failed");
        bail!(format!("Details:\n{}", s));
    } else {
        Ok(())
    }
}
