use assert_cmd::prelude::*;
use std::fs;
use std::path::Path;
use utils;

#[test]
fn build_in_non_crate_directory_doesnt_panic() {
    let fixture = utils::fixture::not_a_crate();
    fixture
        .wasm_pack()
        .arg("build")
        .assert()
        .failure()
        .stderr(predicates::str::contains("missing a `Cargo.toml`"));
}

#[test]
fn it_should_build_js_hello_world_example() {
    let fixture = utils::fixture::js_hello_world();
    fixture.install_local_wasm_bindgen();
    fixture.wasm_pack().arg("build").assert().success();
}

#[test]
fn it_should_build_crates_in_a_workspace() {
    let fixture = utils::fixture::Fixture::new();
    fixture
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                members = ["blah"]
            "#,
        )
        .file(
            Path::new("blah").join("Cargo.toml"),
            r#"
                [package]
                authors = ["The wasm-pack developers"]
                description = "so awesome rust+wasm package"
                license = "WTFPL"
                name = "blah"
                repository = "https://github.com/rustwasm/wasm-pack.git"
                version = "0.1.0"

                [lib]
                crate-type = ["cdylib"]

                [dependencies]
                wasm-bindgen = "=0.2.21"
            "#,
        )
        .file(
            Path::new("blah").join("src").join("lib.rs"),
            r#"
                extern crate wasm_bindgen;
                use wasm_bindgen::prelude::*;

                #[wasm_bindgen]
                pub fn hello() -> u32 { 42 }
            "#,
        )
        .install_local_wasm_bindgen();
    fixture
        .wasm_pack()
        .current_dir(&fixture.path.join("blah"))
        .arg("build")
        .assert()
        .success();
}

#[test]
fn renamed_crate_name_works() {
    let fixture = utils::fixture::Fixture::new();
    fixture
        .readme()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                authors = []

                [lib]
                crate-type = ["cdylib"]
                name = 'bar'

                [dependencies]
                wasm-bindgen = "=0.2.21"
            "#,
        )
        .file(
            "src/lib.rs",
            r#"
                extern crate wasm_bindgen;
                use wasm_bindgen::prelude::*;

                #[wasm_bindgen]
                pub fn one() -> u32 { 1 }
            "#,
        )
        .install_local_wasm_bindgen();
    fixture.wasm_pack().arg("build").assert().success();
}

#[test]
fn it_should_build_nested_project_with_transitive_dependencies() {
    let fixture = utils::fixture::transitive_dependencies();
    fixture.install_local_wasm_bindgen();
    fixture
        .wasm_pack()
        .current_dir(fixture.path.join("main"))
        .arg("build")
        .assert()
        .success();
}

#[test]
fn build_different_profiles() {
    let fixture = utils::fixture::js_hello_world();
    fixture.install_local_wasm_bindgen();

    for profile in ["--dev", "--debug", "--profiling", "--release"]
        .iter()
        .cloned()
    {
        fixture
            .wasm_pack()
            .arg("build")
            .arg(profile)
            .assert()
            .success();
    }
}

#[test]
fn build_with_and_without_wasm_bindgen_debug() {
    for debug in [true, false].iter().cloned() {
        let fixture = utils::fixture::Fixture::new();
        fixture
            .readme()
            .file(
                "Cargo.toml",
                format!(
                    r#"
                    [package]
                    authors = ["The wasm-pack developers"]
                    description = "so awesome rust+wasm package"
                    license = "WTFPL"
                    name = "whatever"
                    repository = "https://github.com/rustwasm/wasm-pack.git"
                    version = "0.1.0"

                    [lib]
                    crate-type = ["cdylib"]

                    [dependencies]
                    wasm-bindgen = "0.2"

                    [package.metadata.wasm-pack.profile.dev.wasm-bindgen]
                    debug-js-glue = {}
                    "#,
                    debug
                ),
            )
            .file(
                "src/lib.rs",
                r#"
                extern crate wasm_bindgen;
                use wasm_bindgen::prelude::*;

                #[wasm_bindgen]
                pub struct MyThing {}

                #[wasm_bindgen]
                impl MyThing {
                    pub fn new() -> MyThing {
                        MyThing {}
                    }

                    pub fn take(self) {}
                }
                "#,
            )
            .install_local_wasm_bindgen();

        fixture
            .wasm_pack()
            .arg("build")
            .arg("--dev")
            .assert()
            .success();

        let contents = fs::read_to_string(fixture.path.join("pkg/whatever.js")).unwrap();
        assert_eq!(
            contents.contains("throw new Error('Attempt to use a moved value')"),
            debug,
            "Should only contain moved value assertions when debug assertions are enabled"
        );
    }
}

#[test]
fn build_with_arbitrary_cargo_options() {
    let fixture = utils::fixture::js_hello_world();
    fixture.install_local_wasm_bindgen();
    fixture
        .wasm_pack()
        .arg("build")
        .arg("--")
        .arg("--no-default-features")
        .assert()
        .success();
}
