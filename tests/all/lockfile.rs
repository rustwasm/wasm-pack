use std::path::Path;
use std::process::Command;
use utils::fixture;
use wasm_pack::{build, lockfile};

#[test]
fn it_gets_wasm_bindgen_version() {
    let fixture = fixture::fixture("tests/fixtures/js-hello-world");
    build_wasm(&fixture.path);
    assert_eq!(
        lockfile::get_wasm_bindgen_version(&fixture.path)
            .unwrap()
            .unwrap(),
        "0.2"
    );
}

#[test]
fn it_gets_wasm_bindgen_version_with_underscores() {
    let fixture = fixture::fixture("tests/fixtures/with-underscores");
    build_wasm(&fixture.path);
    assert_eq!(
        lockfile::get_wasm_bindgen_version(&fixture.path)
            .unwrap()
            .unwrap(),
        "0.2"
    );
}

/// The `step_install_wasm_bindgen` and `step_run_wasm_bindgen` steps only
/// occur after the `step_build_wasm` step. In order to read the lockfile
/// in the test fixture's temporary directory, we should first build the
/// crate, targeting `wasm32-unknown-unknown`.
fn build_wasm(path: &Path) {
    Command::new("cargo")
        .current_dir(path)
        .arg("+nightly")
        .arg("build")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .output()
        .expect("Could not build test fixture's wasm!");
}
