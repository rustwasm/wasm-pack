use std::fs;

fn main() {
    fs::create_dir_all("docs/book/installer").unwrap();
    fs::copy(
        "docs/installer/wasm-pack.js",
        "docs/book/installer/wasm-pack.js",
    ).unwrap();
    let index = fs::read_to_string("docs/installer/index.html").unwrap();
    fs::write(
        "docs/book/installer/index.html",
        fixup(&index),
    ).unwrap();

    let init = fs::read_to_string("docs/installer/init.sh").unwrap();
    fs::write(
        "docs/book/installer/init.sh",
        fixup(&init),
    ).unwrap();
}

fn fixup(input: &str) -> String {
    let manifest = fs::read_to_string("Cargo.toml").unwrap();
    let version = manifest.lines()
        .find(|line| line.starts_with("version ="))
        .unwrap();
    let version = &version[version.find('"').unwrap() + 1..version.rfind('"').unwrap()];

    input.replace("$VERSION", &format!("v{}", version))
}
