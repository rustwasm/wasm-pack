extern crate failure;
extern crate wasm_pack;

use std::fs;

use utils::{self, fixture};
use wasm_pack::readme;

#[test]
fn it_copies_a_readme_default_path() {
    let fixture = fixture::js_hello_world();
    let out_dir = fixture.path.join("pkg");
    fs::create_dir(&out_dir).expect("should create pkg directory OK");

    assert!(readme::copy_from_crate(&fixture.path, &out_dir).is_ok());

    let crate_readme_path = fixture.path.join("README.md");
    let pkg_readme_path = out_dir.join("README.md");
    println!(
        "wasm-pack: should have copied README.md from '{}' to '{}'",
        crate_readme_path.display(),
        pkg_readme_path.display()
    );
    assert!(fs::metadata(&crate_readme_path).is_ok());

    assert!(fs::metadata(&pkg_readme_path).is_ok());

    let crate_readme = utils::file::read_file(&crate_readme_path).unwrap();
    let pkg_readme = utils::file::read_file(&pkg_readme_path).unwrap();
    assert_eq!(crate_readme, pkg_readme);
}

#[test]
fn it_copies_a_readme_provided_path() {
    let fixture = fixture::js_hello_world();
    let out_dir = fixture.path.join("pkg");
    fs::create_dir(&out_dir).expect("should create pkg directory OK");

    assert!(readme::copy_from_crate(&fixture.path, &out_dir).is_ok());
    let crate_readme_path = fixture.path.join("README.md");
    let pkg_readme_path = out_dir.join("README.md");
    println!(
        "wasm-pack: should have copied README.md from '{}' to '{}'",
        crate_readme_path.display(),
        pkg_readme_path.display()
    );
    assert!(fs::metadata(&crate_readme_path).is_ok());
    assert!(fs::metadata(&pkg_readme_path).is_ok());

    let crate_readme = utils::file::read_file(&crate_readme_path).unwrap();
    let pkg_readme = utils::file::read_file(&pkg_readme_path).unwrap();
    assert_eq!(crate_readme, pkg_readme);
}
