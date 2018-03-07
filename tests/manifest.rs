extern crate dir_diff;
extern crate wasm_pack;

use wasm_pack::manifest;

#[test]
fn it_gets_the_crate_name_default_path() {
    assert!(manifest::get_crate_name(".").is_ok());
    assert_eq!(manifest::get_crate_name(".").unwrap(), "wasm-pack");
}

#[test]
fn it_gets_the_crate_name_provided_path() {
    assert!(manifest::get_crate_name("./examples/js-hello-world").is_ok());
    assert_eq!(
        manifest::get_crate_name("./examples/js-hello-world").unwrap(),
        "js-hello-world"
    );
}

#[test]
fn it_creates_a_package_json_default_path() {
    assert!(manifest::write_package_json(".").is_ok());
    assert!(!dir_diff::is_different("./pkg", "./test/fixtures/package-json").unwrap());
}

#[test]
fn it_creates_a_package_json_provided_path() {
    assert!(manifest::write_package_json("./examples/js-hello-world").is_ok());
    assert!(!dir_diff::is_different(
        "./examples/js-hello-world/pkg",
        "./test/fixtures/package-json"
    ).unwrap());
}
