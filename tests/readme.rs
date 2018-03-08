extern crate wasm_pack;

use std::fs;

use wasm_pack::readme;

#[test]
fn it_copies_a_readme_default_path() {
    assert!(readme::copy_from_crate(".").is_ok());
    assert!(fs::metadata("./pkg/README.md").is_ok());
}

#[test]
fn it_creates_a_package_json_provided_path() {
    assert!(readme::copy_from_crate("./examples/js-hello-world").is_ok());
    assert!(fs::metadata("./examples/js-hello-world/pkg/README.md").is_ok());
}
