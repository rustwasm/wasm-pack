//! Building a Rust crate into a `.wasm` binary.

use child;
use emoji;
use error::Error;
use failure::ResultExt;
use progressbar::Step;
use slog::Logger;
use std::path::Path;
use std::process::Command;
use std::str;
use PBAR;

/// Ensure that `rustc` is present and that it is >= 1.30.0
pub fn check_rustc_version(step: &Step) -> Result<String, failure::Error> {
    let msg = format!("{}Checking `rustc` version...", emoji::CRAB);
    PBAR.step(step, &msg);
    let local_minor_version = rustc_minor_version();
    match local_minor_version {
        Some(mv) => {
            if mv < 30 {
              return Err(Error::RustcVersion {
                message: format!(
                  "Your version of Rust, '1.{}', is not supported. Please install Rust version 1.30.0 or higher.",
                  mv.to_string()
                ),
                local_minor_version: mv.to_string(),
              }.into())
            } else {
              Ok(mv.to_string())
            }
      },
      None => Err(Error::RustcMissing {
        message: "We can't figure out what your Rust version is- which means you might not have Rust installed. Please install Rust version 1.30.0 or higher.".to_string(),
      }.into()),
    }
}

// from https://github.com/alexcrichton/proc-macro2/blob/79e40a113b51836f33214c6d00228934b41bd4ad/build.rs#L44-L61
fn rustc_minor_version() -> Option<u32> {
    macro_rules! otry {
        ($e:expr) => {
            match $e {
                Some(e) => e,
                None => return None,
            }
        };
    }
    let output = otry!(Command::new("rustc").arg("--version").output().ok());
    let version = otry!(str::from_utf8(&output.stdout).ok());
    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }
    otry!(pieces.next()).parse().ok()
}

/// Ensure that `rustup` has the `wasm32-unknown-unknown` target installed for
/// current toolchain
pub fn rustup_add_wasm_target(log: &Logger, step: &Step) -> Result<(), failure::Error> {
    let msg = format!("{}Adding WASM target...", emoji::TARGET);
    PBAR.step(step, &msg);
    let mut cmd = Command::new("rustup");
    cmd.arg("target").arg("add").arg("wasm32-unknown-unknown");
    child::run(log, cmd, "rustup")
        .context("Adding the wasm32-unknown-unknown target with rustup")?;
    Ok(())
}

/// Run `cargo build` targetting `wasm32-unknown-unknown`.
pub fn cargo_build_wasm(
    log: &Logger,
    path: &Path,
    debug: bool,
    step: &Step,
) -> Result<(), failure::Error> {
    let msg = format!("{}Compiling to WASM...", emoji::CYCLONE);
    PBAR.step(step, &msg);
    let mut cmd = Command::new("cargo");
    cmd.current_dir(path).arg("build").arg("--lib");
    if !debug {
        cmd.arg("--release");
    }
    cmd.arg("--target").arg("wasm32-unknown-unknown");
    child::run(log, cmd, "cargo build").context("Compiling your crate to WebAssembly")?;
    Ok(())
}

/// Run `cargo build --tests` targetting `wasm32-unknown-unknown`.
pub fn cargo_build_wasm_tests(
    log: &Logger,
    path: &Path,
    debug: bool,
) -> Result<(), failure::Error> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(path).arg("build").arg("--tests");
    if !debug {
        cmd.arg("--release");
    }
    cmd.arg("--target").arg("wasm32-unknown-unknown");
    child::run(log, cmd, "cargo build").context("Compilation of your program failed")?;
    Ok(())
}
