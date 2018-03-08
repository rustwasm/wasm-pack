extern crate wasm_pack;

use std::fs;

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
    let path = ".".to_string();
    wasm_pack::create_pkg_dir(&path).unwrap();
    assert!(manifest::write_package_json(&path).is_ok());
    let package_json_path = format!("{}/pkg/package.json", &path);
    assert!(fs::metadata(package_json_path).is_ok());
}

#[test]
fn it_creates_a_package_json_provided_path() {
    let path = "./examples/js-hello-world".to_string();
    wasm_pack::create_pkg_dir(&path).unwrap();
    assert!(manifest::write_package_json(&path).is_ok());
    let package_json_path = format!("{}/pkg/package.json", &path);
    assert!(fs::metadata(package_json_path).is_ok());
}
