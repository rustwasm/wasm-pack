extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate wasm_pack;

mod utils;

use std::fs;

use wasm_pack::manifest;

#[test]
fn it_gets_the_crate_name_default_path() {
    assert!(manifest::get_crate_name(".").is_ok());
    assert_eq!(manifest::get_crate_name(".").unwrap(), "wasm-pack");
}

#[test]
fn it_gets_the_crate_name_provided_path() {
    assert!(manifest::get_crate_name("tests/fixtures/js-hello-world").is_ok());
    assert_eq!(
        manifest::get_crate_name("tests/fixtures/js-hello-world").unwrap(),
        "js-hello-world"
    );
}

#[test]
fn it_creates_a_package_json_default_path() {
    let path = ".".to_string();
    wasm_pack::command::create_pkg_dir(&path).unwrap();
    assert!(manifest::write_package_json(&path, None).is_ok());
    let package_json_path = format!("{}/pkg/package.json", &path);
    assert!(fs::metadata(package_json_path).is_ok());
    assert!(utils::read_package_json(&path).is_ok());
    let pkg = utils::read_package_json(&path).unwrap();
    assert_eq!(pkg.name, "wasm-pack");
    assert_eq!(pkg.repository.ty, "git");
    assert_eq!(
        pkg.repository.url,
        "https://github.com/ashleygwilliams/wasm-pack.git"
    );
    assert_eq!(pkg.files, ["wasm_pack_bg.wasm"]);
    assert_eq!(pkg.main, "wasm_pack.js");
}

#[test]
fn it_creates_a_package_json_provided_path() {
    let path = "tests/fixtures/js-hello-world".to_string();
    wasm_pack::command::create_pkg_dir(&path).unwrap();
    assert!(manifest::write_package_json(&path, None).is_ok());
    let package_json_path = format!("{}/pkg/package.json", &path);
    assert!(fs::metadata(package_json_path).is_ok());
    assert!(utils::read_package_json(&path).is_ok());
    let pkg = utils::read_package_json(&path).unwrap();
    assert_eq!(pkg.name, "js-hello-world");
}

#[test]
fn it_creates_a_package_json_provided_path_with_scope() {
    let path = "tests/fixtures/scopes".to_string();
    wasm_pack::command::create_pkg_dir(&path).unwrap();
    assert!(manifest::write_package_json(&path, Some("test".to_string())).is_ok());
    let package_json_path = format!("{}/pkg/package.json", &path);
    assert!(fs::metadata(package_json_path).is_ok());
    assert!(utils::read_package_json(&path).is_ok());
    let pkg = utils::read_package_json(&path).unwrap();
    assert_eq!(pkg.name, "@test/scopes-hello-world");
}
