//! Building a Rust crate into a `.wasm` binary.

use child;
use command::build::BuildProfile;
use emoji;
use failure::{Error, ResultExt};
use log::info;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use PBAR;

/// Ensure that `rustc` is present and that it is >= 1.30.0
pub fn check_rustc_version() -> Result<String, Error> {
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

/// Get rustc's sysroot as a PathBuf
fn get_rustc_sysroot() -> Result<PathBuf, Error> {
    let command = Command::new("rustc")
        .args(&["--print", "sysroot"])
        .output()?;

    if command.status.success() {
        Ok(String::from_utf8(command.stdout)?.trim().into())
    } else {
        Err(format_err!(
            "Getting rustc's sysroot wasn't successful. Got {}",
            command.status
        ))
    }
}

/// Checks if the wasm32-unknown-unknown is present in rustc's sysroot.
fn is_wasm32_target_in_sysroot(sysroot: &PathBuf) -> bool {
    let wasm32_target = "wasm32-unknown-unknown";

    let rustlib_path = sysroot.join("lib/rustlib");

    info!("Looking for {} in {:?}", wasm32_target, rustlib_path);

    if rustlib_path.join(wasm32_target).exists() {
        info!("Found {} in {:?}", wasm32_target, rustlib_path);
        true
    } else {
        info!("Failed to find {} in {:?}", wasm32_target, rustlib_path);
        false
    }
}

struct Wasm32Check {
    rustc_path: PathBuf,
    sysroot: PathBuf,
    found: bool,
    is_rustup: bool,
}

impl fmt::Display for Wasm32Check {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let target = "wasm32-unknown-unknown";

        if self.found {
            let rustup_string = if self.is_rustup {
                "It looks like Rustup is being used.".to_owned()
            } else {
                format!("It looks like Rustup is not being used. For non-Rustup setups, the {} target needs to be installed manually. See https://rustwasm.github.io/wasm-pack/book/prerequisites/index.html#target-wasm32-unknown-unknown on how to do this.", target)
            };

            writeln!(
                f,
                "{} target not found in sysroot: {:?}",
                target, self.sysroot
            )
            .and_then(|_| {
                writeln!(
                    f,
                    "\nUsed rustc from the following path: {:?}",
                    self.rustc_path
                )
            })
            .and_then(|_| writeln!(f, "{}", rustup_string))
        } else {
            write!(
                f,
                "sysroot: {:?}, rustc path: {:?}, was found: {}, isRustup: {}",
                self.sysroot, self.rustc_path, self.found, self.is_rustup
            )
        }
    }
}

fn check_wasm32_target() -> Result<Wasm32Check, Error> {
    let sysroot = get_rustc_sysroot()?;
    let rustc_path = which::which("rustc")?;

    // If wasm32-unknown-unknown already exists we're ok.
    if is_wasm32_target_in_sysroot(&sysroot) {
        Ok(Wasm32Check {
            rustc_path,
            sysroot,
            found: true,
            is_rustup: false,
        })
    // If it doesn't exist, then we need to check if we're using rustup.
    } else {
        // If sysroot contains .rustup, then we can assume we're using rustup
        // and use rustup to add the wasm32-unknown-unknown target.
        if sysroot.to_string_lossy().contains(".rustup") {
            rustup_add_wasm_target().map(|()| Wasm32Check {
                rustc_path,
                sysroot,
                found: true,
                is_rustup: true,
            })
        } else {
            Ok(Wasm32Check {
                rustc_path,
                sysroot,
                found: false,
                is_rustup: false,
            })
        }
    }
}

/// Add wasm32-unknown-unknown using `rustup`.
fn rustup_add_wasm_target() -> Result<(), Error> {
    let mut cmd = Command::new("rustup");
    cmd.arg("target").arg("add").arg("wasm32-unknown-unknown");
    child::run(cmd, "rustup").context("Adding the wasm32-unknown-unknown target with rustup")?;

    Ok(())
}

/// Ensure that `rustup` has the `wasm32-unknown-unknown` target installed for
/// current toolchain
pub fn check_for_wasm32_target() -> Result<(), Error> {
    let msg = format!("{}Checking for the Wasm target...", emoji::TARGET);
    PBAR.info(&msg);

    // Check if wasm32 target is present, otherwise bail.
    match check_wasm32_target() {
        Ok(ref wasm32_check) if wasm32_check.found => Ok(()),
        Ok(wasm32_check) => bail!("{}", wasm32_check),
        Err(err) => Err(err),
    }
}

/// Run `cargo build` targetting `wasm32-unknown-unknown`.
pub fn cargo_build_wasm(
    path: &Path,
    profile: BuildProfile,
    extra_options: &Vec<String>,
) -> Result<(), Error> {
    let msg = format!("{}Compiling to Wasm...", emoji::CYCLONE);
    PBAR.info(&msg);
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
