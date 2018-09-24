//! Building a Rust crate into a `.wasm` binary.

use emoji;
use error::Error;
use progressbar::Step;
use std::env;
use std::path::Path;
use std::process::Command;
use std::str;
use PBAR;

/// Ensure that `rustc` is present and that it is >= 1.30.0
pub fn check_rustc_version(step: &Step) -> Result<String, Error> {
    let msg = format!("{}Checking `rustc` version...", emoji::TARGET);
    PBAR.step(step, &msg);
    let local_minor_version = rustc_minor_version();
    match local_minor_version {
        Some(mv) => {
            if mv < 30 {
              return Err(Error::RustcVersion {
                message: format!(
                  "Your version of Rust, '{}', is not supported.",
                  mv.to_string()
                ),
                local_minor_version: mv.to_string(),
              })
            } else {
              Ok(mv.to_string())
            }
      },
      None => Err(Error::RustcMissing {
        message: "We can't figure out what your Rust version is- which means you might not have Rust ins    talled. Please install Rust version 1.30.0 or higher.".to_string(),
      }),
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
    let rustc = otry!(env::var_os("RUSTC"));
    let output = otry!(Command::new(rustc).arg("--version").output().ok());
    let version = otry!(str::from_utf8(&output.stdout).ok());
    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }
    otry!(pieces.next()).parse().ok()
}

/// Ensure that `rustup` has the `wasm32-unknown-unknown` target installed for
/// current toolchain
pub fn rustup_add_wasm_target(step: &Step) -> Result<(), Error> {
    let msg = format!("{}Adding WASM target...", emoji::TARGET);
    PBAR.step(step, &msg);
    let output = Command::new("rustup")
        .arg("target")
        .arg("add")
        .arg("wasm32-unknown-unknown")
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("Adding the wasm32-unknown-unknown target failed", s)
    } else {
        Ok(())
    }
}

/// Run `cargo build` targetting `wasm32-unknown-unknown`.
pub fn cargo_build_wasm(path: &Path, debug: bool, step: &Step) -> Result<(), Error> {
    let msg = format!("{}Compiling to WASM...", emoji::CYCLONE);
    PBAR.step(step, &msg);
    let output = {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(path).arg("build").arg("--lib");
        if !debug {
            cmd.arg("--release");
        }
        cmd.arg("--target").arg("wasm32-unknown-unknown");
        cmd.output()?
    };

    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("Compilation of your program failed", s)
    } else {
        Ok(())
    }
}

/// Run `cargo build --tests` targetting `wasm32-unknown-unknown`.
pub fn cargo_build_wasm_tests(path: &Path, debug: bool) -> Result<(), Error> {
    let output = {
        let mut cmd = Command::new("cargo");
        cmd.current_dir(path).arg("build").arg("--tests");
        if !debug {
            cmd.arg("--release");
        }
        cmd.arg("--target").arg("wasm32-unknown-unknown");
        cmd.output()?
    };

    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("Compilation of your program failed", s)
    } else {
        Ok(())
    }
}
