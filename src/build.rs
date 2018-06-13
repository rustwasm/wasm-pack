use emoji;
use error::Error;
use progressbar::Step;
use std::process::Command;
use PBAR;

pub fn rustup_add_wasm_target(step: &Step) -> Result<(), Error> {
    let msg = format!("{}Adding WASM target...", emoji::TARGET);
    let pb = PBAR.step(step, &msg);
    let output = Command::new("rustup")
        .arg("target")
        .arg("add")
        .arg("wasm32-unknown-unknown")
        .output()?;
    pb.finish();
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("Adding the wasm32-unknown-unknown target failed", s)
    } else {
        Ok(())
    }
}

pub fn cargo_build_wasm(path: &str, debug: bool, step: &Step) -> Result<(), Error> {
    let msg = format!("{}Compiling to WASM...", emoji::CYCLONE);
    let pb = PBAR.step(step, &msg);
    let output = {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(path).arg("build");
        if !debug {
            cmd.arg("--release");
        }
        cmd.arg("--target").arg("wasm32-unknown-unknown");
        cmd.output()?
    };
    pb.finish();
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("Compilation of your program failed", s)
    } else {
        Ok(())
    }
}
