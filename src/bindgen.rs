//! Functionality related to installing and running `wasm-bindgen`.

use binaries::{self, bin_path, install_binaries_from_targz_at_url};
use cargo_metadata;
use child;
use emoji;
use error::Error;
use failure::{self, ResultExt};
use progressbar::Step;
use slog::Logger;
use std::path::{Path, PathBuf};
use std::process::Command;
use target;
use PBAR;

/// Install the `wasm-bindgen` CLI.
///
/// Prefers an existing local install, if any exists. Then checks if there is a
/// global install on `$PATH` that fits the bill. Then attempts to download a
/// tarball from the GitHub releases page, if this target has prebuilt
/// binaries. Finally, falls back to `cargo install`.
pub fn install_wasm_bindgen(
    root_path: &Path,
    version: &str,
    install_permitted: bool,
    step: &Step,
    log: &Logger,
) -> Result<(), failure::Error> {
    // If the `wasm-bindgen` dependency is already met, print a message and return.
    if wasm_bindgen_path(log, root_path)
        .map(|bindgen_path| wasm_bindgen_version_check(&bindgen_path, version, log))
        .unwrap_or(false)
    {
        let msg = format!("{}wasm-bindgen already installed...", emoji::DOWN_ARROW);
        PBAR.step(step, &msg);
        return Ok(());
    }

    // If the `wasm-bindgen` dependency was not met, and installs are not
    // permitted, return a configuration error.
    if !install_permitted {
        let msg = format!("wasm-bindgen v{} is not installed!", version);
        return Err(Error::crate_config(&msg).into());
    }

    let msg = format!("{}Installing wasm-bindgen...", emoji::DOWN_ARROW);
    PBAR.step(step, &msg);

    download_prebuilt_wasm_bindgen(root_path, version).or_else(|e| {
        warn!(
            log,
            "could not download pre-built `wasm-bindgen`: {}. Falling back to `cargo install`.", e
        );
        cargo_install_wasm_bindgen(log, root_path, version)
    })
}

/// Download a tarball containing a pre-built `wasm-bindgen` binary.
pub fn download_prebuilt_wasm_bindgen(
    root_path: &Path,
    version: &str,
) -> Result<(), failure::Error> {
    let target = if target::LINUX && target::x86_64 {
        "x86_64-unknown-linux-musl"
    } else if target::MACOS && target::x86_64 {
        "x86_64-apple-darwin"
    } else if target::WINDOWS && target::x86_64 {
        "x86_64-pc-windows-msvc"
    } else {
        return Err(Error::unsupported(
            "there are no pre-built `wasm-bindgen` binaries for this target",
        )
        .into());
    };

    let url = format!(
        "https://github.com/rustwasm/wasm-bindgen/releases/download/{0}/wasm-bindgen-{0}-{1}.tar.gz",
        version,
        target
    );

    install_binaries_from_targz_at_url(
        root_path,
        &url,
        vec!["wasm-bindgen", "wasm-bindgen-test-runner"],
    )
}

/// Use `cargo install` to install the `wasm-bindgen` CLI locally into the given
/// crate.
pub fn cargo_install_wasm_bindgen(
    logger: &Logger,
    crate_path: &Path,
    version: &str,
) -> Result<(), failure::Error> {
    let mut cmd = Command::new("cargo");
    cmd.arg("install")
        .arg("--force")
        .arg("wasm-bindgen-cli")
        .arg("--version")
        .arg(version)
        .arg("--root")
        .arg(crate_path);

    child::run(logger, cmd, "cargo install").context("Installing wasm-bindgen with cargo")?;
    assert!(binaries::local_bin_path(crate_path, "wasm-bindgen").is_file());
    Ok(())
}

/// Run the `wasm-bindgen` CLI to generate bindings for the current crate's
/// `.wasm`.
pub fn wasm_bindgen_build(
    path: &Path,
    out_dir: &Path,
    name: &str,
    disable_dts: bool,
    target: &str,
    debug: bool,
    step: &Step,
    log: &Logger,
) -> Result<(), failure::Error> {
    let msg = format!("{}Running WASM-bindgen...", emoji::RUNNER);
    PBAR.step(step, &msg);

    let binary_name = name.replace("-", "_");
    let release_or_debug = if debug { "debug" } else { "release" };

    let out_dir = out_dir.to_str().unwrap();

    if let Some(wasm_bindgen_path) = wasm_bindgen_path(log, path) {
        let manifest = path.join("Cargo.toml");
        let target_path = cargo_metadata::metadata(Some(&manifest))
            .unwrap()
            .target_directory;
        let mut wasm_path = PathBuf::from(&target_path)
            .join("wasm32-unknown-unknown")
            .join(release_or_debug)
            .join(binary_name);
        wasm_path.set_extension("wasm");
        let wasm_path = wasm_path.display().to_string();

        let dts_arg = if disable_dts {
            "--no-typescript"
        } else {
            "--typescript"
        };
        let target_arg = match target {
            "nodejs" => "--nodejs",
            "no-modules" => "--no-modules",
            _ => "--browser",
        };
        let bindgen_path = Path::new(&wasm_bindgen_path);
        let mut cmd = Command::new(bindgen_path);
        cmd.current_dir(path)
            .arg(&wasm_path)
            .arg("--out-dir")
            .arg(out_dir)
            .arg(dts_arg)
            .arg(target_arg);

        if debug {
            cmd.arg("--debug");
        }

        child::run(log, cmd, "wasm-bindgen").context("Running the wasm-bindgen CLI")?;
        Ok(())
    } else {
        Err(Error::crate_config("Could not find `wasm-bindgen`").into())
    }
}

/// Check if the `wasm-bindgen` dependency is locally satisfied.
fn wasm_bindgen_version_check(bindgen_path: &PathBuf, dep_version: &str, log: &Logger) -> bool {
    let mut cmd = Command::new(bindgen_path);
    cmd.arg("--version");
    child::run(log, cmd, "wasm-bindgen")
        .map(|stdout| {
            stdout
                .trim()
                .split_whitespace()
                .nth(1)
                .map(|v| {
                    info!(
                        log,
                        "Checking installed `wasm-bindgen` version == expected version: {} == {}",
                        v,
                        dep_version
                    );
                    v == dep_version
                })
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

/// Return a `PathBuf` containing the path to either the local wasm-bindgen
/// version, or the globally installed version if there is no local version.
fn wasm_bindgen_path(log: &Logger, crate_path: &Path) -> Option<PathBuf> {
    bin_path(log, crate_path, "wasm-bindgen")
}

/// Return a `PathBuf` containing the path to either the local
/// wasm-bindgen-test-runner version, or the globally installed version if there
/// is no local version.
pub fn wasm_bindgen_test_runner_path(log: &Logger, crate_path: &Path) -> Option<PathBuf> {
    bin_path(log, crate_path, "wasm-bindgen-test-runner")
}
