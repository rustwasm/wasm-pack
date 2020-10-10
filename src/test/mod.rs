//! Testing a Rust crate compiled to wasm.

pub mod webdriver;

use crate::PBAR;
use child;
use command::global_opts;
use failure::{self, ResultExt};
use std::ffi::OsStr;
use std::path::Path;

/// Run `cargo test` with the `nightly` toolchain and targeting
/// `wasm32-unknown-unknown`.
pub fn cargo_test_wasm<I, K, V>(
    path: &Path,
    release: bool,
    envs: I,
    extra_options: &[String],
) -> Result<(), failure::Error>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    let mut cmd = global_opts().cargo_cmd();

    cmd.envs(envs);
    cmd.current_dir(path).arg("test");

    if PBAR.quiet() {
        cmd.arg("--quiet");
    }

    if release {
        cmd.arg("--release");
    }

    cmd.arg("--target").arg("wasm32-unknown-unknown");

    cmd.args(extra_options);

    child::run(cmd, "cargo test").context("Running Wasm tests with wasm-bindgen-test failed")?;

    // NB: `child::run` took care of ensuring that test output gets printed.
    Ok(())
}
