use assert_cmd::prelude::*;
use predicates::prelude::*;
use utils;

#[test]
fn on_in_release() {
    let fixture = utils::fixture::Fixture::new();
    fixture.readme().cargo_toml("foo").file("src/lib.rs", "");
    fixture.install_local_wasm_bindgen();
    fixture.install_wasm_opt();

    fixture
        .wasm_pack()
        .arg("build")
        .arg("--wat")
        .assert()
        .stderr(predicates::str::contains("wasm-dis"))
        .success();
}
