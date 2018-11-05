use std::path::Path;
use structopt::StructOpt;
use utils;
use wasm_pack::Cli;

#[test]
fn build_in_non_crate_directory_doesnt_panic() {
    let fixture = utils::fixture::not_a_crate();
    let cli = Cli::from_iter_safe(vec![
        "wasm-pack",
        "build",
        &fixture.path.display().to_string(),
    ])
    .unwrap();
    let result = fixture.run(cli.cmd);
    assert!(
        result.is_err(),
        "running wasm-pack in a non-crate directory should fail, but it should not panic"
    );
    let err = result.unwrap_err();
    assert!(err
        .iter_chain()
        .any(|e| e.to_string().contains("missing a `Cargo.toml`")));
}

#[test]
fn it_should_build_js_hello_world_example() {
    let fixture = utils::fixture::js_hello_world();
    fixture.install_local_wasm_bindgen();
    let cli = Cli::from_iter_safe(vec![
        "wasm-pack",
        "build",
        &fixture.path.display().to_string(),
    ])
    .unwrap();
    fixture.run(cli.cmd).unwrap();
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
        );
    fixture.install_local_wasm_bindgen();
    let cli = Cli::from_iter_safe(vec![
        "wasm-pack",
        "build",
        &fixture.path.join("blah").display().to_string(),
    ]).unwrap();
    fixture.run(cli.cmd).unwrap();
}
