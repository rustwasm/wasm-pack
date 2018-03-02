extern crate wasm_pack;

use std::fs;

#[test]
fn it_creates_a_package_json_default_path() {
    assert!(wasm_pack::write_package_json(".").is_ok());
    assert!(fs::metadata("./pkg/package.json").is_ok());
}

#[test]
fn it_creates_a_package_json_provided_path() {
    assert!(wasm_pack::write_package_json("./examples/js-hello-world").is_ok());
    assert!(fs::metadata("./pkg/package.json").is_ok());
}
