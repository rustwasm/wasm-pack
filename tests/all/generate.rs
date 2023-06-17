use crate::utils;
use assert_cmd::prelude::*;

#[test]
fn new_with_no_name_errors() {
    let fixture = utils::fixture::not_a_crate();
    fixture.install_local_cargo_generate();
    fixture.wasm_pack().arg("new").assert().failure();
}

#[test]
fn new_with_name_succeeds() {
    let fixture = utils::fixture::not_a_crate();
    fixture.install_local_cargo_generate();
    fixture
        .wasm_pack()
        .arg("new")
        .arg("hello")
        .assert()
        .success();
}
