extern crate failure;
extern crate wasm_pack;

use std::fs;

use utils;
use wasm_pack::readme;

#[test]
fn it_copies_a_readme_default_path() {
    let fixture = utils::fixture(".");
    fs::create_dir(fixture.path.join("pkg")).expect("should create pkg directory OK");

    let step = wasm_pack::progressbar::Step::new(1);
    assert!(readme::copy_from_crate(&fixture.path, &step).is_ok());
    let crate_readme_path = fixture.path.join("README.md");
    let pkg_readme_path = fixture.path.join("pkg").join("README.md");
    assert!(fs::metadata(&pkg_readme_path).is_ok());
    let crate_readme = utils::readme::read_file(&crate_readme_path).unwrap();
    let pkg_readme = utils::readme::read_file(&pkg_readme_path).unwrap();
    assert_eq!(crate_readme, pkg_readme);
}

#[test]
fn it_creates_a_package_json_provided_path() {
    let fixture = utils::fixture("tests/fixtures/js-hello-world");
    fs::create_dir(fixture.path.join("pkg")).expect("should create pkg directory OK");

    let step = wasm_pack::progressbar::Step::new(1);
    assert!(readme::copy_from_crate(&fixture.path, &step).is_ok());
    let crate_readme_path = fixture.path.join("README.md");
    let pkg_readme_path = fixture.path.join("pkg").join("README.md");
    assert!(fs::metadata(&pkg_readme_path).is_ok());
    let crate_readme = utils::readme::read_file(&crate_readme_path).unwrap();
    let pkg_readme = utils::readme::read_file(&pkg_readme_path).unwrap();
    assert_eq!(crate_readme, pkg_readme);
}
