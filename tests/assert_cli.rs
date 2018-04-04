extern crate assert_cli;

#[test]
fn check_output_pass() {
    assert_cli::Assert::main_binary()
        .with_args(&["init", "tests/fixtures/assert-cli"])
        .stdout()
        .contains(":-) Done in")
        .stdout()
        .contains(":-) Your WASM pkg is ready to publish at tests/fixtures/assert-cli/pkg")
        .stderr()
        .contains("Finished dev [unoptimized + debuginfo] target(s) in")
        .stderr()
        .contains("Running `target/debug/wasm-pack init tests/fixtures/assert-cli`")
        .unwrap();
}
