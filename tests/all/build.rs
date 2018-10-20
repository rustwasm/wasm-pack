use structopt::StructOpt;
use utils;
use wasm_pack::{command, logger, Cli};

#[test]
fn build_in_non_crate_directory_doesnt_panic() {
    let fixture = utils::fixture::not_a_crate();
    let cli = Cli::from_iter_safe(vec![
        "wasm-pack",
        "build",
        &fixture.path.display().to_string(),
    ])
    .unwrap();
    let logger = logger::new(&cli.cmd, cli.verbosity).unwrap();
    let result = command::run_wasm_pack(cli.cmd, &logger);
    assert!(
        result.is_err(),
        "running wasm-pack in a non-crate directory should fail, but it should not panic"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("missing a `Cargo.toml`"));
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
    let logger = logger::new(&cli.cmd, cli.verbosity).unwrap();
    command::run_wasm_pack(cli.cmd, &logger)
        .expect("running wasm-pack in a js-hello-world directory should succeed.");
}
