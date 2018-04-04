use PBAR;
use console::style;
use emoji;
use std::process::Command;

pub fn rustup_add_wasm_target() {
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
        .output()
        .unwrap_or_else(|e| panic!("{} failed to execute process: {}", emoji::ERROR, e));
    pb.finish();
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);

        print!(
            "{}  rustup_add_wasm_target failed and stderr was:\n{}",
            emoji::ERROR,
            s
        );
    }
}

pub fn cargo_build_wasm(path: &str) {
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
        .output()
        .unwrap_or_else(|e| panic!("{} failed to execute process: {}", emoji::ERROR, e));
    pb.finish();
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);

        print!(
            "{}  cargo_build_wasm failed and stderr was:\n{}",
            emoji::ERROR,
            s
        );
    }
}
