//! Building a Rust crate into a `.wasm` binary.

use child;
use command::build::BuildProfile;
use emoji;
use failure::{Error, ResultExt};
use progressbar::Step;
use slog::Logger;
use std::path::Path;
use std::process::Command;
use std::str;
use PBAR;

/// Ensure that `rustc` is present and that it is >= 1.30.0
pub fn check_rustc_version(step: &Step) -> Result<String, Error> {
    let msg = format!("{}Checking `rustc` version...", emoji::CRAB);
    PBAR.step(step, &msg);
    let local_minor_version = rustc_minor_version();
    match local_minor_version {
        Some(mv) => {
            if mv < 30 {
                bail!(
                    "Your version of Rust, '1.{}', is not supported. Please install Rust version 1.30.0 or higher.",
                    mv.to_string()
                )
            } else {
                Ok(mv.to_string())
            }
        },
        None => bail!("We can't figure out what your Rust version is- which means you might not have Rust installed. Please install Rust version 1.30.0 or higher."),
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
pub fn rustup_add_wasm_target(log: &Logger, step: &Step) -> Result<(), Error> {
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
    profile: BuildProfile,
    step: &Step,
    extra_options: &Vec<String>,
) -> Result<(), Error> {
    let msg = format!("{}Compiling to WASM...", emoji::CYCLONE);
    PBAR.step(step, &msg);
    let mut cmd = Command::new("cargo");
    cmd.current_dir(path).arg("build").arg("--lib");
    match profile {
        BuildProfile::Profiling => {
            // Once there are DWARF debug info consumers, force enable debug
            // info, because builds that use the release cargo profile disables
            // debug info.
            //
            // cmd.env("RUSTFLAGS", "-g");
            cmd.arg("--release");
        }
        BuildProfile::Release => {
            cmd.arg("--release");
        }
        BuildProfile::Dev => {
            // Plain cargo builds use the dev cargo profile, which includes
            // debug info by default.
        }
    }
    cmd.arg("--target").arg("wasm32-unknown-unknown");
    cmd.args(extra_options);
    child::run(log, cmd, "cargo build").context("Compiling your crate to WebAssembly failed")?;
    Ok(())
}

/// Run `cargo build --tests` targetting `wasm32-unknown-unknown`.
pub fn cargo_build_wasm_tests(log: &Logger, path: &Path, debug: bool) -> Result<(), Error> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(path).arg("build").arg("--tests");
    if !debug {
        cmd.arg("--release");
    }
    cmd.arg("--target").arg("wasm32-unknown-unknown");
    child::run(log, cmd, "cargo build").context("Compilation of your program failed")?;
    Ok(())
}
