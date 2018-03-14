extern crate failure;
extern crate wasm_pack;

mod utils;

use std::fs;

use wasm_pack::readme;

#[test]
fn it_copies_a_readme_default_path() {
    let path = ".".to_string();
    assert!(readme::copy_from_crate(&path).is_ok());
    let crate_readme_path = format!("{}/README.md", &path);
    let pkg_readme_path = format!("{}/pkg/README.md", &path);
    assert!(fs::metadata(&pkg_readme_path).is_ok());
    let crate_readme = utils::read_file(&crate_readme_path).unwrap();
    let pkg_readme = utils::read_file(&pkg_readme_path).unwrap();
    assert_eq!(crate_readme, pkg_readme);
}

#[test]
fn it_creates_a_package_json_provided_path() {
    let path = "tests/fixtures/js-hello-world".to_string();
    assert!(readme::copy_from_crate(&path).is_ok());
    let crate_readme_path = format!("{}/README.md", &path);
    let pkg_readme_path = format!("{}/pkg/README.md", &path);
    assert!(fs::metadata(&pkg_readme_path).is_ok());
    let crate_readme = utils::read_file(&crate_readme_path).unwrap();
    let pkg_readme = utils::read_file(&pkg_readme_path).unwrap();
    assert_eq!(crate_readme, pkg_readme);
}
