use emoji;
use std::process::Command;

pub fn npm_pack(path: &str) {
    let pkg_file_path = format!("{}/pkg", path);
    let output = Command::new("npm")
        .current_dir(pkg_file_path)
        .arg("pack")
        .output()
        .unwrap_or_else(|e| panic!("{} failed to execute process: {}", emoji::ERROR, e));
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        print!("{}  npm_pack failed and stderr was:\n{}", emoji::ERROR, s);
    }
}

pub fn npm_publish(path: &str) {
    let pkg_file_path = format!("{}/pkg", path);
    let output = Command::new("npm")
        .current_dir(pkg_file_path)
        .arg("publish")
        .output()
        .unwrap_or_else(|e| panic!("{} failed to execute process: {}", emoji::ERROR, e));
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        print!(
            "{}  npm_publish failed and stderr was:\n{}",
            emoji::ERROR,
            s
        );
    }
}
