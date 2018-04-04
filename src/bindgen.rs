use PBAR;
use console::style;
use emoji;
use std::process::Command;

pub fn cargo_install_wasm_bindgen() {
    let step = format!(
        "{} {}Installing WASM-bindgen...",
        style("[6/7]").bold().dim(),
        emoji::DOWN_ARROW
    );
    let pb = PBAR.message(&step);
    let _output = Command::new("cargo")
        .arg("install")
        .arg("wasm-bindgen")
        .output()
        .unwrap_or_else(|e| panic!("{} failed to execute process: {}", emoji::ERROR, e));
    pb.finish();
}

pub fn wasm_bindgen_build(path: &str, name: &str) {
    let step = format!(
        "{} {}Running WASM-bindgen...",
        style("[7/7]").bold().dim(),
        emoji::RUNNER
    );
    let pb = PBAR.message(&step);
    let binary_name = name.replace("-", "_");
    let wasm_path = format!("target/wasm32-unknown-unknown/release/{}.wasm", binary_name);
    let _output = Command::new("wasm-bindgen")
        .current_dir(path)
        .arg(&wasm_path)
        .arg("--out-dir")
        .arg("./pkg")
        .output()
        .unwrap_or_else(|e| panic!("{} failed to execute process: {}", emoji::ERROR, e));
    pb.finish();
}
