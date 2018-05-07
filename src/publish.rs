use error::Error;
use std::process::Command;

pub fn npm_publish(path: &str) -> Result<(), Error> {
    let pkg_file_path = format!("{}/pkg", path);
    let output = Command::new("npm")
        .current_dir(pkg_file_path)
        .arg("publish")
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("Publishing to npm failed", s)
    } else {
        Ok(())
    }
}
