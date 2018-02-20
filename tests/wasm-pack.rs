extern crate wasm_pack;

use std::fs;

#[test]
fn it_creates_a_package_json() {
    assert!(wasm_pack::write_package_json().is_ok());
    assert!(fs::metadata("./pkg/package.json").is_ok());
}
