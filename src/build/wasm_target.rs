//! Checking for the wasm32 target

use crate::child;
use crate::emoji;
use crate::PBAR;
use anyhow::{anyhow, bail, Context, Result};
use log::info;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Wasm32Check {
    rustc_path: PathBuf,
    sysroot: PathBuf,
    found: bool,
    is_rustup: bool,
}

impl fmt::Display for Wasm32Check {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let target = "wasm32-unknown-unknown";

        if !self.found {
            let rustup_string = if self.is_rustup {
                "It looks like Rustup is being used.".to_owned()
            } else {
                format!("It looks like Rustup is not being used. For non-Rustup setups, the {} target needs to be installed manually. See https://rustwasm.github.io/wasm-pack/book/prerequisites/non-rustup-setups.html on how to do this.", target)
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

/// Ensure that `rustup` has the `wasm32-unknown-unknown` target installed for
/// current toolchain
pub fn check_for_wasm32_target() -> Result<()> {
    let msg = format!("{}Checking for the Wasm target...", emoji::TARGET);
    PBAR.info(&msg);

    // Check if wasm32 target is present, otherwise bail.
    match check_wasm32_target() {
        Ok(None) => Ok(()),
        Ok(Some(wasm32_check)) => bail!("{}", wasm32_check),
        Err(err) => Err(err),
    }
}

/// Get rustc's sysroot as a PathBuf
fn get_rustc_sysroot() -> Result<PathBuf> {
    let command = Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()?;

    if command.status.success() {
        Ok(String::from_utf8(command.stdout)?.trim().into())
    } else {
        Err(anyhow!(
            "Getting rustc's sysroot wasn't successful. Got {}",
            command.status
        ))
    }
}

/// Checks if the wasm32-unknown-unknown is present in rustc's sysroot.
fn is_wasm32_target_in_sysroot(sysroot: &Path) -> bool {
    let wasm32_target = "wasm32-unknown-unknown";

    let mut path = sysroot.join("lib/rustlib");

    info!("Looking for {} in {:?}", wasm32_target, path);
    path.push(wasm32_target);

    if path.exists() {
        info!("Found {} in {:?}", wasm32_target, path.parent());
        true
    } else {
        info!("Failed to find {} in {:?}", wasm32_target, path.parent());
        false
    }
}

fn check_wasm32_target() -> Result<Option<Wasm32Check>> {
    let sysroot = get_rustc_sysroot()?;

    // If wasm32-unknown-unknown already exists we're ok.
    if is_wasm32_target_in_sysroot(&sysroot) {
        Ok(None)
    // If it doesn't exist, then we need to check if we're using rustup.
    } else {
        // If sysroot contains "rustup", then we can assume we're using rustup
        // and use rustup to add the wasm32-unknown-unknown target.
        if sysroot.to_string_lossy().contains("rustup") {
            rustup_add_wasm_target().map(|()| None)
        } else {
            Ok(Some(Wasm32Check {
                rustc_path: which::which("rustc")?,
                sysroot,
                found: false,
                is_rustup: false,
            }))
        }
    }
}

/// Add wasm32-unknown-unknown using `rustup`.
fn rustup_add_wasm_target() -> Result<()> {
    let mut cmd = Command::new("rustup");
    cmd.args(["target", "add", "wasm32-unknown-unknown"]);
    child::run(cmd, "rustup").context("Adding the wasm32-unknown-unknown target with rustup")?;

    Ok(())
}
