use console::style;
use emoji;
use error::Error;
use std::process::Command;

pub fn cargo_build_wasm(path: &str) -> Result<(), Error> {
    let output = Command::new("cargo")
        .current_dir(path)
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("Compilation of your program failed", s)
    } else {
        Ok(())
    }
}
