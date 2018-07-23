extern crate failure;
extern crate wasm_pack;

mod utils;

use std::fs;
use std::path::PathBuf;

use wasm_pack::readme;

#[test]
fn it_copies_a_readme_default_path() {
    let step = wasm_pack::progressbar::Step::new(1);
    let path = PathBuf::from(".");
    assert!(readme::copy_from_crate(&path, &step).is_ok());
    let crate_readme_path = &path.join("README.md");
    let pkg_readme_path = &path.join("pkg").join("README.md");
    assert!(fs::metadata(&pkg_readme_path).is_ok());
    let crate_readme = utils::read_file(&crate_readme_path).unwrap();
    let pkg_readme = utils::read_file(&pkg_readme_path).unwrap();
    assert_eq!(crate_readme, pkg_readme);
}

#[test]
fn it_creates_a_package_json_provided_path() {
    let step = wasm_pack::progressbar::Step::new(1);
    let path = PathBuf::from("tests/fixtures/js-hello-world");
    assert!(readme::copy_from_crate(&path, &step).is_ok());
    let crate_readme_path = &path.join("README.md");
    let pkg_readme_path = &path.join("pkg").join("README.md");
    assert!(fs::metadata(&pkg_readme_path).is_ok());
    let crate_readme = utils::read_file(&crate_readme_path).unwrap();
    let pkg_readme = utils::read_file(&pkg_readme_path).unwrap();
    assert_eq!(crate_readme, pkg_readme);
}
