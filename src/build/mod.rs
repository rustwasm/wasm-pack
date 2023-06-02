//! Building a Rust crate into a `.wasm` binary.

use crate::child;
use crate::command::build::BuildProfile;
use crate::emoji;
use crate::manifest;
use crate::PBAR;
use anyhow::{anyhow, bail, Context, Result};
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;
use std::str;

pub mod wasm_target;

/// Used when comparing the currently installed
/// wasm-pack version with the latest on crates.io.
pub struct WasmPackVersion {
    /// The currently installed wasm-pack version.
    pub local: &'static str,
    /// The latest version of wasm-pack that's released at
    /// crates.io.
    pub latest: Vec<u8>,
}

/// Ensure that `rustc` is present and that it is >= 1.30.0
pub fn check_rustc_version() -> Result<String> {
    try_check_rustc_version().unwrap_or_else(|| bail!("We can't figure out what your Rust version is- which means you might not have Rust installed. Please install Rust version 1.30.0 or higher."))
}

// from https://github.com/alexcrichton/proc-macro2/blob/79e40a113b51836f33214c6d00228934b41bd4ad/build.rs#L44-L61
fn try_check_rustc_version() -> Option<Result<String>> {
    let output = Command::new("rustc").arg("--version").output().ok()?.stdout;
    let minor = str::from_utf8(
        output
            .strip_prefix(b"rustc 1.")?
            .split(|&b| b == b'.')
            .next()?,
    )
    .ok()?;
    let supported = match minor.len() {
        2 => minor >= "30",
        1 => false,
        _ => true,
    };
    Some(if supported {
        Ok(minor.to_owned())
    } else {
        Err(anyhow!("Your version of Rust, '1.{}', is not supported. Please install Rust version 1.30.0 or higher.", minor))
    })
}

/// Checks and returns local and latest versions of wasm-pack
pub fn check_wasm_pack_versions() -> Result<WasmPackVersion> {
    match wasm_pack_local_version() {
        Some(local) => Ok(WasmPackVersion {local, latest: manifest::return_wasm_pack_latest_version()?.unwrap_or_else(|| Vec::new())}),
        None => bail!("We can't figure out what your wasm-pack version is, make sure the installation path is correct.")
    }
}

fn wasm_pack_local_version() -> Option<&'static str> {
    let output = env!("CARGO_PKG_VERSION");
    Some(output)
}

/// Run `cargo build` targetting `wasm32-unknown-unknown`.
pub fn cargo_build_wasm(
    path: &Path,
    profile: BuildProfile,
    extra_options: &[OsString],
) -> Result<()> {
    let msg = format!("{}Compiling to Wasm...", emoji::CYCLONE);
    PBAR.info(&msg);

    let mut cmd = Command::new("cargo");
    cmd.current_dir(path).arg("build").arg("--lib");

    if PBAR.quiet() {
        cmd.arg("--quiet");
    }

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
    child::run(cmd, "cargo build").context("Compiling your crate to WebAssembly failed")?;
    Ok(())
}

/// Runs `cargo build --tests` targeting `wasm32-unknown-unknown`.
///
/// This generates the `Cargo.lock` file that we use in order to know which version of
/// wasm-bindgen-cli to use when running tests.
///
/// Note that the command to build tests and the command to run tests must use the same parameters, i.e. features to be
/// disabled / enabled must be consistent for both `cargo build` and `cargo test`.
///
/// # Parameters
///
/// * `path`: Path to the crate directory to build tests.
/// * `debug`: Whether to build tests in `debug` mode.
/// * `extra_options`: Additional parameters to pass to `cargo` when building tests.
pub fn cargo_build_wasm_tests(path: &Path, debug: bool, extra_options: &[String]) -> Result<()> {
    let mut cmd = Command::new("cargo");

    cmd.current_dir(path).arg("build").arg("--tests");

    if PBAR.quiet() {
        cmd.arg("--quiet");
    }

    if !debug {
        cmd.arg("--release");
    }

    cmd.arg("--target").arg("wasm32-unknown-unknown");

    cmd.args(extra_options);

    child::run(cmd, "cargo build").context("Compilation of your program failed")?;
    Ok(())
}
