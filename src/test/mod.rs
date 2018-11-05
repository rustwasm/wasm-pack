//! Testing a Rust crate compiled to wasm.

pub mod webdriver;

use child;
use failure::{self, ResultExt};
use log::info;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

/// Run `cargo test` with the `nightly` toolchain and targeting
/// `wasm32-unknown-unknown`.
pub fn cargo_test_wasm<I, K, V>(path: &Path, release: bool, envs: I) -> Result<(), failure::Error>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    let output = {
        let mut cmd = Command::new("cargo");
        cmd.envs(envs);
        cmd.current_dir(path).arg("test");
        if release {
            cmd.arg("--release");
        }
        cmd.arg("--target").arg("wasm32-unknown-unknown");
        child::run(cmd, "cargo test").context("Running Wasm tests with wasm-bindgen-test failed")?
    };

    for line in output.lines() {
        info!("test output: {}", line);
        println!("{}", line);
    }
    Ok(())
}
