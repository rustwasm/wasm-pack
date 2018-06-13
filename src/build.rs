use console::style;
use emoji;
use error::Error;
use std::process::Command;
use PBAR;

pub fn rustup_add_wasm_target() -> Result<(), Error> {
    let step = format!(
        "{} {}Adding WASM target...",
        style("[1/7]").bold().dim(),
        emoji::TARGET
    );
    PBAR.message(&step)?;
    let output = Command::new("rustup")
        .arg("target")
        .arg("add")
        .arg("wasm32-unknown-unknown")
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("Adding the wasm32-unknown-unknown target failed", s)
    } else {
        Ok(())
    }
}

pub fn cargo_build_wasm(path: &str, debug: bool) -> Result<(), Error> {
    let step = format!(
        "{} {}Compiling to WASM...",
        style("[2/7]").bold().dim(),
        emoji::CYCLONE
    );
    PBAR.message(&step)?;
    let output = {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(path).arg("build");
        if !debug {
            cmd.arg("--release");
        }
        cmd.arg("--target").arg("wasm32-unknown-unknown");
        cmd.output()?
    };
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("Compilation of your program failed", s)
    } else {
        Ok(())
    }
}
