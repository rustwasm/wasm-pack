#[path = "../tests/all/utils/fixture.rs"]
#[allow(unused)]
mod fixture;

use std::process::Stdio;

fn run_wasm_pack() {
    let fixture = fixture::dual_license();
    assert!(fixture
        .wasm_pack()
        .arg("build")
        .arg("--mode")
        .arg("no-install")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap()
        .success())
}

fn parse_crates_io() {
    assert_eq!(Some(&b"0.11.0"[..]), wasm_pack::manifest::get_max_version(br#"{"categories":[{"category":"WebAssembly","crates_cnt":1319,"created_at":"2018-03-01T16:00:11.531177+00:00","description":"Crates for use when targeting WebAssembly, or for manipulating WebAssembly.","id":"wasm","slug":"wasm"}],"crate":{"badges":[],"categories":["wasm"],"created_at":"2018-03-16T08:37:12.179096+00:00","description":" your favorite rust -> wasm workflow tool!","documentation":"https://rustwasm.github.io/wasm-pack/","downloads":572316,"exact_match":false,"homepage":null,"id":"wasm-pack","keywords":[],"links":{"owner_team":"/api/v1/crates/wasm-pack/owner_team","owner_user":"/api/v1/crates/wasm-pack/owner_user","owners":"/api/v1/crates/wasm-pack/owners","reverse_dependencies":"/api/v1/crates/wasm-pack/reverse_dependencies","version_downloads":"/api/v1/crates/wasm-pack/downloads","versions":null},"max_stable_version":"0.11.0","max_version":"0.11.0","name":"wasm-pack","newest_version":"0.11.0","recent_downloads":57531,"repository":"https://github.com/rustwasm/wasm-pack.git","updated_at":"2023-03-19T18:34:09.441463+00:00","versions":[753886,566082,469444,421933,396623,210640,208425,143211,142828,139463,128362,111637,109520,101086,99719,97024,95449,94906,90427,85070]},"keywords":[]}]}"#));
}

iai::main!(run_wasm_pack, parse_crates_io);
