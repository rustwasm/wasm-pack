use console::style;
use emoji;
use error::Error;
use std::process::Command;

pub fn wasm_bindgen_build(path: &str, name: &str) -> Result<(), Error> {
    let binary_name = name.replace("-", "_");
    let wasm_path = format!("target/wasm32-unknown-unknown/release/{}.wasm", binary_name);
    let output = Command::new("wasm-bindgen")
        .current_dir(path)
        .arg(&wasm_path)
        .arg("--out-dir")
        .arg("./pkg")
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("wasm-bindgen failed to execute properly", s)
    } else {
        Ok(())
    }
}
