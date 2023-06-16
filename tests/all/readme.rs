extern crate anyhow;
extern crate wasm_pack;

use std::fs;

use crate::utils::{self, fixture};
use assert_cmd::prelude::*;
use predicates::boolean::PredicateBooleanExt;
use wasm_pack::manifest::CrateData;
use wasm_pack::readme;

#[test]
fn it_copies_a_readme_default_path() {
    let fixture = fixture::js_hello_world();
    let out_dir = fixture.path.join("pkg");
    fs::create_dir(&out_dir).expect("should create pkg directory OK");
    let crate_data = CrateData::new(&fixture.path, None).unwrap();

    assert!(readme::copy_from_crate(&crate_data, &fixture.path, &out_dir).is_ok());

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
    let fixture = fixture::Fixture::new();
    fixture
        .hello_world_src_lib()
        .file(
            "Cargo.toml",
            r#"
            [package]
            authors = ["The wasm-pack developers"]
            description = "so awesome rust+wasm package"
            license = "WTFPL"
            name = "js-hello-world"
            readme = "docs/README.md"
            repository = "https://github.com/rustwasm/wasm-pack.git"
            version = "0.1.0"

            [lib]
            crate-type = ["cdylib"]

            [dependencies]
            # Note that this uses and `=` dependency because there are
            # various tests which assert that the version of wasm
            # bindgen downloaded is what we expect, and if `=` is
            # removed then it will download whatever the newest version
            # of wasm-bindgen is which may not be what's listed here.
            wasm-bindgen = "=0.2.74"

            [dev-dependencies]
            wasm-bindgen-test = "0.3"
        "#,
        )
        .file(
            "docs/README.md",
            r#"
            # Fixture!
            > an example rust -> wasm project
        "#,
        );

    let crate_docs_dir = fixture.path.join("docs");
    let out_dir = fixture.path.join("pkg");
    fs::create_dir(&out_dir).expect("should create pkg directory OK");
    let crate_data = CrateData::new(&fixture.path, None).unwrap();

    assert!(readme::copy_from_crate(&crate_data, &fixture.path, &out_dir).is_ok());
    let crate_readme_path = crate_docs_dir.join("README.md");
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
fn it_ignores_a_disabled_readme() {
    let fixture = fixture::Fixture::new();
    fixture
        .hello_world_src_lib()
        .file(
            "Cargo.toml",
            r#"
            [package]
            authors = ["The wasm-pack developers"]
            description = "so awesome rust+wasm package"
            name = "js-hello-world"
            readme = false
            repository = "https://github.com/rustwasm/wasm-pack.git"
            version = "0.1.0"

            [lib]
            crate-type = ["cdylib"]

            [dependencies]
            # Note that this uses and `=` dependency because there are
            # various tests which assert that the version of wasm
            # bindgen downloaded is what we expect, and if `=` is
            # removed then it will download whatever the newest version
            # of wasm-bindgen is which may not be what's listed here.
            wasm-bindgen = "=0.2.74"

            [dev-dependencies]
            wasm-bindgen-test = "0.3"
        "#,
        )
        .license()
        .wasm_pack()
        .arg("build")
        .assert()
        .success()
        .stderr(predicates::str::contains("origin crate has no README").not());
}
