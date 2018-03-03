extern crate wasm_pack;

use std::fs;

#[test]
fn it_gets_the_crate_name_default_path() {
    assert!(wasm_pack::get_crate_name(".").is_ok());
    assert_eq!(wasm_pack::get_crate_name(".").unwrap(), "wasm-pack");
}

#[test]
fn it_gets_the_crate_name_provided_path() {
    assert!(wasm_pack::get_crate_name("./examples/js-hello-world").is_ok());
    assert_eq!(
        wasm_pack::get_crate_name("./examples/js-hello-world").unwrap(),
        "js-hello-world"
    );
}

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
