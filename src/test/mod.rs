//! Testing a Rust crate compiled to wasm.

pub mod webdriver;

use crate::child;
use crate::PBAR;
use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

/// Run `cargo test` with the `nightly` toolchain and targeting
/// `wasm32-unknown-unknown`.
pub fn cargo_test_wasm<I, K, V>(
    path: &Path,
    release: bool,
    envs: I,
    extra_options: &[String],
) -> Result<()>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    let mut cmd = Command::new("cargo");

    cmd.envs(envs);
    cmd.current_dir(path).args(
        std::iter::empty::<&OsStr>()
            .chain(["test".as_ref()])
            .chain(PBAR.quiet().then_some("--quiet".as_ref()))
            .chain(release.then_some("--release".as_ref()))
            .chain(["--target".as_ref(), "wasm32-unknown-unknown".as_ref()])
            .chain(extra_options.iter().map(|s| s.as_ref())),
    );

    child::run(cmd, "cargo test").context("Running Wasm tests with wasm-bindgen-test failed")?;

    // NB: `child::run` took care of ensuring that test output gets printed.
    Ok(())
}
