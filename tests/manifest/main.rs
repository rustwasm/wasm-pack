extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate wasm_pack;

mod utils;

use std::fs;

use utils::FailureExt;
use wasm_pack::manifest;

#[test]
fn it_gets_the_crate_name_default_path() {
    assert!(manifest::get_crate_name(".").is_ok());
    assert_eq!(manifest::get_crate_name(".").unwrap_pretty(), "wasm-pack");
}

#[test]
fn it_gets_the_crate_name_provided_path() {
    assert!(manifest::get_crate_name("tests/fixtures/js-hello-world").is_ok());
    assert_eq!(
        manifest::get_crate_name("tests/fixtures/js-hello-world").unwrap_pretty(),
        "js-hello-world"
    );
}

#[test]
fn it_creates_a_package_json_default_path() {
    let path = ".".to_string();
    wasm_pack::command::create_pkg_dir(&path).unwrap_pretty();
    utils::mock_wasm(path.as_ref(), "wasm_pack");
    assert!(manifest::write_package_json(&path, None).is_ok());
    let package_json_path = format!("{}/pkg/package.json", &path);
    assert!(fs::metadata(package_json_path).is_ok());
    assert!(utils::read_package_json(&path).is_ok());
    let pkg = utils::read_package_json(&path).unwrap_pretty();
    assert_eq!(pkg.name, "wasm-pack");
    assert_eq!(pkg.repository.ty, "git");
    assert_eq!(
        pkg.repository.url,
        "https://github.com/ashleygwilliams/wasm-pack.git"
    );
    assert_eq!(pkg.files, ["wasm_pack_bg.wasm"]);
    assert_eq!(pkg.main, "wasm_pack.js");
    assert!(pkg.dependencies.is_none());
}

#[test]
fn it_creates_a_package_json_provided_path() {
    let path = "tests/fixtures/js-hello-world".to_string();
    wasm_pack::command::create_pkg_dir(&path).unwrap_pretty();
    utils::mock_wasm(path.as_ref(), "js_hello_world");
    assert!(manifest::write_package_json(&path, None).is_ok());
    let package_json_path = format!("{}/pkg/package.json", &path);
    assert!(fs::metadata(package_json_path).is_ok());
    assert!(utils::read_package_json(&path).is_ok());
    let pkg = utils::read_package_json(&path).unwrap_pretty();
    assert_eq!(pkg.name, "js-hello-world");
}

#[test]
fn it_creates_a_package_json_provided_path_with_scope() {
    let path = "tests/fixtures/scopes".to_string();
    wasm_pack::command::create_pkg_dir(&path).unwrap_pretty();
    utils::mock_wasm(path.as_ref(), "scopes_hello_world");
    manifest::write_package_json(&path, Some("test".to_string())).unwrap_pretty();
    let package_json_path = format!("{}/pkg/package.json", &path);
    assert!(fs::metadata(package_json_path).is_ok());
    assert!(utils::read_package_json(&path).is_ok());
    let pkg = utils::read_package_json(&path).unwrap_pretty();
    assert_eq!(pkg.name, "@test/scopes-hello-world");
}

#[test]
fn it_creates_package_json_with_dependencies() {
    let path = "tests/fixtures/dependencies".to_string();
    wasm_pack::command::create_pkg_dir(&path).unwrap_pretty();
    utils::mock_wasm_with_deps(path.as_ref(), "dependencies");
    manifest::write_package_json(&path, None).unwrap_pretty();
    let package_json_path = format!("{}/pkg/package.json", &path);
    assert!(fs::metadata(package_json_path).is_ok());
    assert!(utils::read_package_json(&path).is_ok());
    let pkg = utils::read_package_json(&path).unwrap_pretty();
    let deps = pkg.dependencies.as_ref().unwrap();
    assert_eq!(deps.len(), 1);
    let (k, v) = deps.iter().next().unwrap();
    assert_eq!(k, "foo");
    assert_eq!(v, "^1.0.2");
}
