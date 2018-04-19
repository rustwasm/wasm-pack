use PBAR;
use console::style;
use emoji;
use failure::Error;
use std::process::Command;

pub fn rustup_add_wasm_target() -> Result<(), Error> {
    let step = format!(
        "{} {}Adding WASM target...",
        style("[1/7]").bold().dim(),
        emoji::TARGET
    );
    let pb = PBAR.message(&step);
    let output = Command::new("rustup")
        .arg("target")
        .arg("add")
        .arg("wasm32-unknown-unknown")
        .output()?;
    pb.finish();
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        PBAR.error("Adding the wasm32-unknown-unknown target failed");
        bail!(format!("Details:\n{}", s));
    } else {
        Ok(())
    }
}

pub fn cargo_build_wasm(path: &str) -> Result<(), Error> {
    let step = format!(
        "{} {}Compiling to WASM...",
        style("[2/7]").bold().dim(),
        emoji::CYCLONE
    );
    let pb = PBAR.message(&step);
    let output = Command::new("cargo")
        .current_dir(path)
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .output()?;
    pb.finish();
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        PBAR.error("Compilation of your program failed");
        bail!(format!("Details:\n{}", s));
    } else {
        Ok(())
    }
}
