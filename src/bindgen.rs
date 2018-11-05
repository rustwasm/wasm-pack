//! Functionality related to installing and running `wasm-bindgen`.

use child;
use command::build::BuildProfile;
use emoji;
use failure::{self, ResultExt};
use log::debug;
use log::{info, warn};
use manifest::CrateData;
use progressbar::Step;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use target;
use wasm_pack_binary_install::{Cache, Download};
use which::which;
use PBAR;

/// Install the `wasm-bindgen` CLI.
///
/// Prefers an existing local install, if any exists. Then checks if there is a
/// global install on `$PATH` that fits the bill. Then attempts to download a
/// tarball from the GitHub releases page, if this target has prebuilt
/// binaries. Finally, falls back to `cargo install`.
pub fn install_wasm_bindgen(
    cache: &Cache,
    version: &str,
    install_permitted: bool,
    step: &Step,
) -> Result<Download, failure::Error> {
    // If `wasm-bindgen` is installed globally and it has the right version, use
    // that. Assume that other tools are installed next to it.
    //
    // This situation can arise if `wasm-bindgen` is already installed via
    // `cargo install`, for example.
    if let Ok(path) = which("wasm-bindgen") {
        debug!("found global wasm-bindgen binary at: {}", path.display());
        if wasm_bindgen_version_check(&path, version) {
            return Ok(Download::at(path.parent().unwrap()));
        }
    }

    let msg = format!("{}Installing wasm-bindgen...", emoji::DOWN_ARROW);
    PBAR.step(step, &msg);

    let dl = download_prebuilt_wasm_bindgen(&cache, version, install_permitted);
    match dl {
        Ok(dl) => return Ok(dl),
        Err(e) => {
            warn!(
                "could not download pre-built `wasm-bindgen`: {}. Falling back to `cargo install`.",
                e
            );
        }
    }

    cargo_install_wasm_bindgen(&cache, version, install_permitted)
}

/// Downloads a precompiled copy of wasm-bindgen, if available.
pub fn download_prebuilt_wasm_bindgen(
    cache: &Cache,
    version: &str,
    install_permitted: bool,
) -> Result<Download, failure::Error> {
    let url = match prebuilt_url(version) {
        Some(url) => url,
        None => bail!("no prebuilt wasm-bindgen binaries are available for this platform"),
    };
    let binaries = &["wasm-bindgen", "wasm-bindgen-test-runner"];
    match cache.download(install_permitted, "wasm-bindgen", binaries, &url)? {
        Some(download) => Ok(download),
        None => bail!("wasm-bindgen v{} is not installed!", version),
    }
}

/// Returns the URL of a precompiled version of wasm-bindgen, if we have one
/// available for our host platform.
fn prebuilt_url(version: &str) -> Option<String> {
    let target = if target::LINUX && target::x86_64 {
        "x86_64-unknown-linux-musl"
    } else if target::MACOS && target::x86_64 {
        "x86_64-apple-darwin"
    } else if target::WINDOWS && target::x86_64 {
        "x86_64-pc-windows-msvc"
    } else {
        return None;
    };

    Some(format!(
        "https://github.com/rustwasm/wasm-bindgen/releases/download/{0}/wasm-bindgen-{0}-{1}.tar.gz",
        version,
        target
    ))
}

/// Use `cargo install` to install the `wasm-bindgen` CLI locally into the given
/// crate.
pub fn cargo_install_wasm_bindgen(
    cache: &Cache,
    version: &str,
    install_permitted: bool,
) -> Result<Download, failure::Error> {
    let dirname = format!("wasm-bindgen-cargo-install-{}", version);
    let destination = cache.join(dirname.as_ref());
    if destination.exists() {
        return Ok(Download::at(&destination));
    }

    if !install_permitted {
        bail!("wasm-bindgen v{} is not installed!", version)
    }

    // Run `cargo install` to a temporary location to handle ctrl-c gracefully
    // and ensure we don't accidentally use stale files in the future
    let tmp = cache.join(format!(".{}", dirname).as_ref());
    drop(fs::remove_dir_all(&tmp));
    fs::create_dir_all(&tmp)?;

    let mut cmd = Command::new("cargo");
    cmd.arg("install")
        .arg("--force")
        .arg("wasm-bindgen-cli")
        .arg("--version")
        .arg(version)
        .arg("--root")
        .arg(&tmp);

    child::run(cmd, "cargo install").context("Installing wasm-bindgen with cargo")?;

    fs::rename(&tmp, &destination)?;
    Ok(Download::at(&destination))
}

/// Run the `wasm-bindgen` CLI to generate bindings for the current crate's
/// `.wasm`.
pub fn wasm_bindgen_build(
    data: &CrateData,
    bindgen: &Download,
    out_dir: &Path,
    disable_dts: bool,
    target: &str,
    profile: BuildProfile,
    step: &Step,
) -> Result<(), failure::Error> {
    let msg = format!("{}Running WASM-bindgen...", emoji::RUNNER);
    PBAR.step(step, &msg);

    let release_or_debug = match profile {
        BuildProfile::Release | BuildProfile::Profiling => "release",
        BuildProfile::Dev => "debug",
    };

    let out_dir = out_dir.to_str().unwrap();

    let wasm_path = data
        .target_directory()
        .join("wasm32-unknown-unknown")
        .join(release_or_debug)
        .join(data.crate_name())
        .with_extension("wasm");

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
    let bindgen_path = bindgen.binary("wasm-bindgen");
    let mut cmd = Command::new(bindgen_path);
    cmd.arg(&wasm_path)
        .arg("--out-dir")
        .arg(out_dir)
        .arg(dts_arg)
        .arg(target_arg);

    let profile = data.configured_profile(profile);
    if profile.wasm_bindgen_debug_js_glue() {
        cmd.arg("--debug");
    }
    if !profile.wasm_bindgen_demangle_name_section() {
        cmd.arg("--no-demangle");
    }
    if profile.wasm_bindgen_dwarf_debug_info() {
        cmd.arg("--keep-debug");
    }

    child::run(cmd, "wasm-bindgen").context("Running the wasm-bindgen CLI")?;
    Ok(())
}

/// Check if the `wasm-bindgen` dependency is locally satisfied.
fn wasm_bindgen_version_check(bindgen_path: &PathBuf, dep_version: &str) -> bool {
    let mut cmd = Command::new(bindgen_path);
    cmd.arg("--version");
    child::run(cmd, "wasm-bindgen")
        .map(|stdout| {
            stdout
                .trim()
                .split_whitespace()
                .nth(1)
                .map(|v| {
                    info!(
                        "Checking installed `wasm-bindgen` version == expected version: {} == {}",
                        v, dep_version
                    );
                    v == dep_version
                })
                .unwrap_or(false)
        })
        .unwrap_or(false)
}
