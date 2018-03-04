use std::process::Command;

pub fn rustup_add_wasm_target() {
    let output = Command::new("rustup")
        .arg("target")
        .arg("add")
        .arg("wasm32-unknown-unknown")
        .output()
        .unwrap_or_else(|e| panic!("failed to execute process: {}", e));

    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout);

        println!(
            "✅ rustup_add_wasm_target succeeded and stdout was:\n{}",
            s
        );
    } else {
        let s = String::from_utf8_lossy(&output.stderr);

        print!("⛔  rustup_add_wasm_target failed and stderr was:\n{}", s);
    }
}

pub fn cargo_build_wasm(path: &str) {
    let output = Command::new("cargo")
        .current_dir(path)
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .output()
        .unwrap_or_else(|e| panic!("failed to execute process: {}", e));

    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout);

        println!("✅  cargo_build_wasm succeeded and stdout was:\n{}", s);
        println!("🏎️ 💨  compiled to wasm!");
    } else {
        let s = String::from_utf8_lossy(&output.stderr);

        print!("⛔  cargo_build_wasm failed and stderr was:\n{}", s);
    }
}
