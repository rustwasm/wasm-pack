//! Functionality related to installing and running `wasm-bindgen`.

use curl;
use emoji;
use error::Error;
use failure;
use flate2;
use progressbar::Step;
use slog::Logger;
use std::ffi;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar;
use which::which;
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
) -> Result<(), Error> {
    // If the `wasm-bindgen` dependency is already met, print a message and return.
    if wasm_bindgen_path(root_path)
        .map(|bindgen_path| wasm_bindgen_version_check(&bindgen_path, version))
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
        return Error::crate_config(&msg);
    }

    let msg = format!("{}Installing wasm-bindgen...", emoji::DOWN_ARROW);
    PBAR.step(step, &msg);

    download_prebuilt_wasm_bindgen(root_path, version).or_else(|e| {
        warn!(
            log,
            "could not download pre-built `wasm-bindgen`: {}. Falling back to `cargo install`.", e
        );
        cargo_install_wasm_bindgen(root_path, version)
    })
}

fn curl(url: &str) -> Result<Vec<u8>, failure::Error> {
    let mut data = Vec::new();

    fn with_url_context<T, E>(url: &str, r: Result<T, E>) -> Result<T, impl failure::Fail>
    where
        Result<T, E>: failure::ResultExt<T, E>,
    {
        use failure::ResultExt;
        r.with_context(|_| format!("when requesting {}", url))
    }

    let mut easy = curl::easy::Easy::new();
    with_url_context(url, easy.follow_location(true))?;
    with_url_context(url, easy.url(url))?;

    {
        let mut transfer = easy.transfer();
        with_url_context(
            url,
            transfer.write_function(|part| {
                data.extend_from_slice(part);
                Ok(part.len())
            }),
        )?;
        with_url_context(url, transfer.perform())?;
    }

    let code = with_url_context(url, easy.response_code())?;
    if 200 <= code && code < 300 {
        Ok(data)
    } else {
        Err(Error::http(&format!(
            "received a bad HTTP status code ({}) when requesting {}",
            code, url
        )).into())
    }
}

/// Download a tarball containing a pre-built `wasm-bindgen` binary.
pub fn download_prebuilt_wasm_bindgen(root_path: &Path, version: &str) -> Result<(), Error> {
    let linux = cfg!(target_os = "linux");
    let macos = cfg!(target_os = "macos");
    let windows = cfg!(windows);
    let x86_64 = cfg!(target_arch = "x86_64");

    let target = if linux && x86_64 {
        "x86_64-unknown-linux-musl"
    } else if macos && x86_64 {
        "x86_64-apple-darwin"
    } else if windows && x86_64 {
        "x86_64-pc-windows-msvc"
    } else {
        return Err(Error::unsupported(
            "there are no pre-built `wasm-bindgen` binaries for this target",
        ));
    };

    let url = format!(
        "https://github.com/rustwasm/wasm-bindgen/releases/download/{0}/wasm-bindgen-{0}-{1}.tar.gz",
        version,
        target
    );

    let tarball = curl(&url).map_err(|e| Error::http(&e.to_string()))?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(&tarball[..]));

    let bin = root_path.join("bin");
    fs::create_dir_all(&bin)?;

    let mut found_wasm_bindgen = false;
    let mut found_test_runner = false;

    for entry in archive.entries()? {
        let mut entry = entry?;

        let dest = match entry.path()?.file_stem() {
            Some(f) if f == ffi::OsStr::new("wasm-bindgen") => {
                found_wasm_bindgen = true;
                bin.join(entry.path()?.file_name().unwrap())
            }
            Some(f) if f == ffi::OsStr::new("wasm-bindgen-test-runner") => {
                found_test_runner = true;
                bin.join(entry.path()?.file_name().unwrap())
            }
            _ => continue,
        };

        entry.unpack(dest)?;
    }

    if found_wasm_bindgen && found_test_runner {
        Ok(())
    } else {
        Err(Error::archive(
            "the wasm-bindgen tarball was missing expected executables",
        ))
    }
}

/// Use `cargo install` to install the `wasm-bindgen` CLI to the given root
/// path.
pub fn cargo_install_wasm_bindgen(root_path: &Path, version: &str) -> Result<(), Error> {
    let output = Command::new("cargo")
        .arg("install")
        .arg("--force")
        .arg("wasm-bindgen-cli")
        .arg("--version")
        .arg(version)
        .arg("--root")
        .arg(root_path)
        .output()?;
    if !output.status.success() {
        let message = "Installing wasm-bindgen failed".to_string();
        let s = String::from_utf8_lossy(&output.stderr);
        Err(Error::Cli {
            message,
            stderr: s.to_string(),
        })
    } else {
        if cfg!(target_os = "windows") {
            assert!(root_path.join("bin").join("wasm-bindgen.exe").is_file());
        } else {
            assert!(root_path.join("bin").join("wasm-bindgen").is_file());
        }
        Ok(())
    }
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
) -> Result<(), Error> {
    let msg = format!("{}Running WASM-bindgen...", emoji::RUNNER);
    PBAR.step(step, &msg);

    let binary_name = name.replace("-", "_");
    let release_or_debug = if debug { "debug" } else { "release" };

    let out_dir = out_dir.to_str().unwrap();

    if let Some(wasm_bindgen_path) = wasm_bindgen_path(path) {
        let wasm_path = format!(
            "target/wasm32-unknown-unknown/{}/{}.wasm",
            release_or_debug, binary_name
        );
        let dts_arg = if disable_dts {
            "--no-typescript"
        } else {
            "--typescript"
        };
        let target_arg = match target {
            "nodejs" => "--nodejs",
            _ => "--browser",
        };
        let bindgen_path = Path::new(&wasm_bindgen_path);
        let output = Command::new(bindgen_path)
            .current_dir(path)
            .arg(&wasm_path)
            .arg("--out-dir")
            .arg(out_dir)
            .arg(dts_arg)
            .arg(target_arg)
            .output()?;
        if !output.status.success() {
            let s = String::from_utf8_lossy(&output.stderr);
            Error::cli("wasm-bindgen failed to execute properly", s)
        } else {
            Ok(())
        }
    } else {
        Error::crate_config("Could not find `wasm-bindgen`")
    }
}

/// Check if the `wasm-bindgen` dependency is locally satisfied.
fn wasm_bindgen_version_check(bindgen_path: &PathBuf, dep_version: &str) -> bool {
    Command::new(bindgen_path)
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .split_whitespace()
                .nth(1)
                .map(|v| v == dep_version)
                .unwrap_or(false)
        }).unwrap_or(false)
}

/// Return a `PathBuf` containing the path to either the local wasm-bindgen
/// version, or the globally installed version if there is no local version.
fn wasm_bindgen_path(crate_path: &Path) -> Option<PathBuf> {
    // Return the path to the local `wasm-bindgen`, if it exists.
    let local_bindgen_path = |crate_path: &Path| -> Option<PathBuf> {
        let mut p = crate_path.to_path_buf();
        p.push("bin");
        if cfg!(target_os = "windows") {
            p.push("wasm-bindgen.exe");
        } else {
            p.push("wasm-bindgen");
        }
        if p.is_file() {
            Some(p)
        } else {
            None
        }
    };

    // Return the path to the global `wasm-bindgen`, if it exists.
    let global_bindgen_path = || -> Option<PathBuf> {
        if let Ok(p) = which("wasm-bindgen") {
            Some(p)
        } else {
            None
        }
    };

    local_bindgen_path(crate_path).or_else(global_bindgen_path)
}
