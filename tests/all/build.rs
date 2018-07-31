use structopt::StructOpt;
use utils;
use wasm_pack::{command, logger, Cli};

#[test]
fn build_in_non_crate_directory_doesnt_panic() {
    let fixture = utils::fixture::fixture("tests/fixtures/not-a-crate");
    let cli = Cli::from_iter_safe(vec![
        "wasm-pack",
        "build",
        &fixture.path.display().to_string(),
    ]).unwrap();
    let logger = logger::new(&cli.cmd, cli.verbosity).unwrap();
    assert!(
        command::run_wasm_pack(cli.cmd, &logger).is_err(),
        "running wasm-pack in a non-crate directory should fail, but it should not panic"
    );
}
