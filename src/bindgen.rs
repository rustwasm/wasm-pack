use std::process::Command;

pub fn cargo_install_wasm_bindgen() {
    let output = Command::new("cargo")
        .arg("install")
        .arg("--git")
        .arg("https://github.com/alexcrichton/wasm-bindgen")
        .output()
        .unwrap_or_else(|e| panic!("failed to execute process: {}", e));

    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout);

        println!(
            "✅  cargo_install_wasm_bindgen succeeded and stdout was:\n{}",
            s
        );
    } else {
        let s = String::from_utf8_lossy(&output.stderr);

        print!(
            "⛔  cargo_install_wasm_bindgen failed and stderr was:\n{}",
            s
        );
    }
}

pub fn wasm_bindgen_build(path: &str, name: &str) {
    let binary_name = name.replace("-", "_");
    let path_to_wasm = format!("target/wasm32-unknown-unknown/release/{}.wasm", binary_name);
    let output = Command::new("wasm-bindgen")
        .current_dir(path)
        .arg(&path_to_wasm)
        .arg("--out-dir")
        .arg("./pkg")
        .output()
        .unwrap_or_else(|e| panic!("failed to execute process: {}", e));

    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout);

        println!("✅  wasm_bindgen_build succeeded and stdout was:\n{}", s);
        println!("🏄<200d>♀️  ready to go!");
    } else {
        let s = String::from_utf8_lossy(&output.stderr);

        print!("⛔  wasm_bindgen_build failed and stderr was:\n{}", s);
    }
}
