//! Functionality related to installing and running `wasm-bindgen`.

use binary_install::{Cache, Download};
use child;
use command::build::BuildProfile;
use emoji;
use failure::{self, ResultExt};
use log::debug;
use log::{info, warn};
use manifest::CrateData;
use progressbar::Step;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use target;
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

    // Otherwise, attempt to install wasm_bindgen using a pre-built binary
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

    // Finally, if all else fails, install wasm_bindgen using cargo
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
    debug!(
        "Attempting to use a `cargo install`ed version of `wasm-bindgen={}`",
        version
    );

    let cargo_install_dirname = format!("wasm-bindgen-cargo-install-{}", version);
    let cargo_install_cache_dirname = cache.join(cargo_install_dirname.as_ref());
    let destination_dl = Download::at(&cargo_install_cache_dirname);

    if let Ok(bindgen_path) = destination_dl.binary("wasm-bindgen") {
        debug!(
            "`cargo install`ed `wasm-bindgen={}` already exists at {}",
            version,
            bindgen_path.display()
        );
        return Ok(destination_dl);
    }

    if !install_permitted {
        bail!("wasm-bindgen v{} is not installed!", version)
    }

    let tmp_dirname = cargo_install_to_tmp_dir(&cargo_install_dirname, &cargo_install_cache_dirname, version)?;

    // `cargo install` will put the installed binaries in `$root/bin/*`, but we
    // just want them in `$root/*` directly (which matches how the tarballs are
    // laid out, and where the rest of our code expects them to be). So we do a
    // little renaming here.
    for f in ["wasm-bindgen", "wasm-bindgen-test-runner"].iter().cloned() {
        let from = tmp_dirname
            .join("bin")
            .join(f)
            .with_extension(env::consts::EXE_EXTENSION);
        let to = tmp_dirname.join(from.file_name().unwrap());
        fs::rename(&from, &to).with_context(|_| {
            format!(
                "failed to move {} to {} for `cargo install`ed `wasm-bindgen`",
                from.display(),
                to.display()
            )
        })?;
    }

    // Remove destination directory if already exists,
    // e.g. if only the bindgen binary does not exist.
    if cargo_install_cache_dirname.exists() {
        fs::remove_dir_all(&cargo_install_cache_dirname).with_context(|_| {
            format!(
                "failed to remove existing bindgen cache directory: {}",
                cargo_install_cache_dirname.display()
            )
        })?;
    }

    // Finally, move the `tmp` directory into our binary cache.
    fs::rename(&tmp_dirname, &cargo_install_cache_dirname)?;

    Ok(destination_dl)
}

// Run `cargo install` to a temporary location to handle ctrl-c gracefully
// and ensure we don't accidentally use stale files in the future
fn cargo_install_to_tmp_dir(bin_dirname: &str, cache_root_dirname: &PathBuf, version: &str) -> Result<PathBuf, failure::Error> {
    let tmp = cache_root_dirname.join(format!(".{}", bin_dirname));
    drop(fs::remove_dir_all(&tmp));
    debug!(
        "cargo installing wasm-bindgen to tempdir: {}",
        tmp.display()
    );
    fs::create_dir_all(&tmp)
        .context("failed to create temp dir for `cargo install wasm-bindgen`")?;

    let mut cmd = Command::new("cargo");
    cmd.arg("install")
        .arg("--force")
        .arg("wasm-bindgen-cli")
        .arg("--version")
        .arg(version)
        .arg("--root")
        .arg(&tmp);

    child::run(cmd, "cargo install").context("Installing wasm-bindgen with cargo")?;
    Ok(tmp)
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
    let bindgen_path = bindgen.binary("wasm-bindgen")?;
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
