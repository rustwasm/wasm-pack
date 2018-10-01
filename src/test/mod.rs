//! Testing a Rust crate compiled to wasm.

pub mod webdriver;

use error::Error;
use slog::Logger;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

/// Run `cargo test` with the `nightly` toolchain and targeting
/// `wasm32-unknown-unknown`.
pub fn cargo_test_wasm<I, K, V>(
    path: &Path,
    release: bool,
    log: &Logger,
    envs: I,
) -> Result<(), Error>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    use std::sync::Mutex;
    lazy_static! {
        static ref ONE_TEST_AT_A_TIME: Mutex<()> = Mutex::new(());
    }
    let _locked = ONE_TEST_AT_A_TIME.lock().unwrap();

    let output = {
        let mut cmd = Command::new("cargo");
        cmd.envs(envs);
        cmd.current_dir(path).arg("test");
        if release {
            cmd.arg("--release");
        }
        cmd.arg("--target").arg("wasm32-unknown-unknown");
        cmd.output()?
    };

    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("Running wasm tests failed", s)
    } else {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            info!(log, "test output: {}", line);
            println!("{}", line);
        }
        Ok(())
    }
}
