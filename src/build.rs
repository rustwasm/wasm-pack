//! Building a Rust crate into a `.wasm` binary.

use child;
use command::build::BuildProfile;
use emoji;
use failure::{Error, ResultExt};
use log::info;
use progressbar::Step;
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;
use std::string::FromUtf8Error;
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
        }
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

/// Get rustc's sysroot as a String
fn get_rustc_sysroot() -> Result<String, FromUtf8Error> {
    String::from_utf8(
        Command::new("rustc")
            .args(&["--print", "sysroot"])
            .stdout(Stdio::piped())
            .output()
            .unwrap()
            .stdout,
    )
}

/// Checks if the wasm32-unknown-unknown is present in rustc's sysroot.
fn is_wasm32_target_in_sysroot(sysroot: &str) -> Result<bool, Error> {
    let wasm32_target = "wasm32-unknown-unknown";

    info!("Looking for {} in {}", wasm32_target, sysroot);

    let rustlib_path = &format!("{}/lib/rustlib", sysroot.replace("\n", ""));

    let ls_command = Command::new("ls")
        .arg(rustlib_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let grep_command_status = Command::new("grep")
        .args(&["-x", wasm32_target])
        .stdin(ls_command.stdout.unwrap())
        .status();

    match grep_command_status {
        Ok(status) if status.success() => {
            info!("Found {} in {}", wasm32_target, sysroot);
            Ok(true)
        }
        _ => {
            info!("Failed to find {} in {}", wasm32_target, sysroot);
            Ok(false)
        }
    }
}

fn check_wasm32_target() -> Result<bool, Error> {
    let sysroot = get_rustc_sysroot()?;

    // If wasm32-unknown-unknown already exists we're ok.
    match is_wasm32_target_in_sysroot(&sysroot) {
        Ok(true) => Ok(true),
        // If it doesn't exist, then we need to check if we're using rustup.
        _ => {
            // If sysroot contains .rustup, then we can assume we're using rustup
            // and use rustup to add the wasm32-unknown-unknown target.
            if sysroot.contains(".rustup") {
                rustup_add_wasm_target()
            } else {
                Ok(false)
            }
        }
    }
}

/// Add wasm32-unknown-unknown using `rustup`.
fn rustup_add_wasm_target() -> Result<bool, Error> {
    let mut cmd = Command::new("rustup");
    cmd.arg("target").arg("add").arg("wasm32-unknown-unknown");
    child::run(cmd, "rustup").context("Adding the wasm32-unknown-unknown target with rustup")?;

    Ok(true)
}

/// Ensure that `rustup` has the `wasm32-unknown-unknown` target installed for
/// current toolchain
pub fn check_for_wasm32_target(step: &Step) -> Result<(), Error> {
    let msg = format!("{}Checking for the Wasm target...", emoji::TARGET);
    PBAR.step(step, &msg);

    // Check if wasm32 target is present, otherwise bail.
    match check_wasm32_target() {
        Ok(true) => Ok(()),
        _ => bail!("wasm32-unknown-unknown target not found!"),
    }
}

/// Run `cargo build` targetting `wasm32-unknown-unknown`.
pub fn cargo_build_wasm(
    path: &Path,
    profile: BuildProfile,
    step: &Step,
    extra_options: &Vec<String>,
) -> Result<(), Error> {
    let msg = format!("{}Compiling to Wasm...", emoji::CYCLONE);
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
    child::run(cmd, "cargo build").context("Compiling your crate to WebAssembly failed")?;
    Ok(())
}

/// Run `cargo build --tests` targetting `wasm32-unknown-unknown`.
///
/// This generates the `Cargo.lock` file that we use in order to know which version of
/// wasm-bindgen-cli to use when running tests.
pub fn cargo_build_wasm_tests(path: &Path, debug: bool) -> Result<(), Error> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(path).arg("build").arg("--tests");
    if !debug {
        cmd.arg("--release");
    }
    cmd.arg("--target").arg("wasm32-unknown-unknown");
    child::run(cmd, "cargo build").context("Compilation of your program failed")?;
    Ok(())
}
