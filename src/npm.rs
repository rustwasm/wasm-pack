use failure::Error;
use std::process::Command;
use PBAR;

pub fn npm_pack(path: &str) -> Result<(), Error> {
    let pkg_file_path = format!("{}/pkg", path);
    let output = Command::new("npm")
        .current_dir(pkg_file_path)
        .arg("pack")
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        PBAR.error("npm_pack failed");
        bail!(format!("stderr was {}", s));
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
        PBAR.error("npm_publish failed");
        bail!(format!("stderr was {}", s));
    } else {
        Ok(())
    }
}