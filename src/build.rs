//! Building a Rust crate into a `.wasm` binary.

use cargo_metadata;
use cargo_metadata::Message;
use emoji;
use error::Error;
use progressbar::Step;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::str;
use tempfile::NamedTempFile;
use PBAR;

/// Ensure that `rustc` is present and that it is >= 1.30.0
pub fn check_rustc_version(step: &Step) -> Result<String, Error> {
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
                });
            } else {
                Ok(mv.to_string())
            }
        }
        None => Err(Error::RustcMissing {
            message: "We can't figure out what your Rust version is- which means you might not have Rust installed. Please install Rust version 1.30.0 or higher.".to_string(),
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
///
/// Returns the location of the built wasm file.
pub fn cargo_build_wasm(
    path: &Path,
    debug: bool,
    step: &Step,
    crate_name: &str,
) -> Result<PathBuf, Error> {
    let msg = format!("{}Compiling to WASM...", emoji::CYCLONE);
    PBAR.step(step, &msg);

    // Since pipes like `Stdio::piped()` have a fixed capacity, we could deadlock with us
    // waiting on stdout and cargo blocking on the full stderr pipe. This is why we use a file
    // here
    let mut stderr = NamedTempFile::new()?;

    let mut cmd = Command::new("cargo");
    cmd.current_dir(path)
        .arg("+nightly")
        .arg("build")
        .arg("--lib")
        .arg("--target=wasm32-unknown-unknown")
        .arg("--message-format=json")
        .stderr(Stdio::from(stderr.reopen()?))
        .stdout(Stdio::piped());
    if !debug {
        cmd.arg("--release");
    }
    let mut output = cmd.spawn()?;

    let message_stream = output
        .stdout
        .take()
        .expect("cargo child process should always have an stdout");

    let mut wasm_file = None;

    for message in cargo_metadata::parse_message_stream(message_stream) {
        match message.unwrap() {
            Message::CompilerArtifact(artifact) => {
                if artifact.package_id.name() == crate_name {
                    let pos = artifact
                        .target
                        .crate_types
                        .iter()
                        .position(|x| x == "cdylib");

                    if let Some(pos) = pos {
                        wasm_file = Some(PathBuf::from(&artifact.filenames[pos]));
                    };
                };
            }
            Message::CompilerMessage(message) => {
                eprintln!(
                    "{}",
                    message
                        .message
                        .rendered
                        .unwrap_or_else(|| "Unrendered Message".to_string())
                );
            }
            _ => (),
        };
    }

    let status = output.wait()?;
    if !status.success() {
        let mut errors = String::new();
        stderr.read_to_string(&mut errors)?;
        Err(Error::Cli {
            message: "Compilation of your program failed".to_string(),
            stderr: errors,
        })
    } else {
        if let Some(wasm_file) = wasm_file {
            Ok(wasm_file)
        } else {
            Err(Error::CrateConfig {
                message: "Your crate didn't produce a cdylib".to_string(),
            })
        }
    }
}

/// Run `cargo build --tests` targetting `wasm32-unknown-unknown`.
pub fn cargo_build_wasm_tests(path: &Path, debug: bool) -> Result<(), Error> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(path)
        .arg("build")
        .arg("--tests")
        .arg("--target=wasm32-unknown-unknown");
    if !debug {
        cmd.arg("--release");
    }

    let output = cmd.output()?;

    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("Compilation of your program failed", s)
    } else {
        Ok(())
    }
}
