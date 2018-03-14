use std::process::Command;
use console::style;
use emoji;
use progressbar;

pub fn cargo_install_wasm_bindgen() {
    let step = format!(
        "{} {}Installing WASM-bindgen...",
        style("[6/7]").bold().dim(),
        emoji::DOWN_ARROW
    );
    let pb = progressbar::new(step);
    let _output = Command::new("cargo")
        .arg("install")
        .arg("--git")
        .arg("https://github.com/alexcrichton/wasm-bindgen")
        .output()
        .unwrap_or_else(|e| panic!("{} failed to execute process: {}", emoji::ERROR, e));
    pb.finish();
    //if !output.status.success() {
    //    let s = String::from_utf8_lossy(&output.stderr);

    //      print!(
    //         "{}  cargo_install_wasm_bindgen failed and stderr was:\n{}",
    //         emoji::ERROR,
    //         s
    //     );
    // }
}

pub fn wasm_bindgen_build(path: &str, name: &str) {
    let step = format!(
        "{} {}Running WASM-bindgen...",
        style("[7/7]").bold().dim(),
        emoji::RUNNER
    );
    let pb = progressbar::new(step);
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
    //if !output.status.success() {
    //    let s = String::from_utf8_lossy(&output.stderr);

    //    print!("  wasm_bindgen_build failed and stderr was:\n{}", emoji::ERROR, s);
    //}
}
