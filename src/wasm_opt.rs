//! Support for downloading and executing `wasm-opt`

use crate::child;
use crate::install;
use crate::PBAR;
use anyhow::{anyhow, Result};
use binary_install::{Cache, Download};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Indicate where we found the `wasm-opt` executable.
pub enum WasmOptLocation {
    /// We discovered a `wasm-opt` binary already installed.
    Installed(PathBuf),
    /// We downloaded `wasm-opt` successfully.
    Downloaded(Download),
    /// No download was requested.
    NoDownloadRequested,
    /// This platform is not supported by `wasm-opt`.
    PlatformNotSupported,
}

impl WasmOptLocation {
    /// Return the location of the `wasm-opt` binary.
    pub fn binary(&self) -> Result<PathBuf> {
        match self {
            WasmOptLocation::Installed(installed_at) => Ok(installed_at.to_path_buf()),
            WasmOptLocation::Downloaded(d) => d.binary("bin/wasm-opt"),
            WasmOptLocation::NoDownloadRequested => {
                Err(anyhow!("Skipping wasm-opt as no downloading was requested"))
            }
            WasmOptLocation::PlatformNotSupported => Err(anyhow!(
                "Skipping wasm-opt because it is not supported on this platform"
            )),
        }
    }
}

/// Execute `wasm-opt` over wasm binaries found in `out_dir`, downloading if
/// necessary into `cache`. Passes `args` to each invocation of `wasm-opt`.
pub fn run(cache: &Cache, out_dir: &Path, args: &[String], install_permitted: bool) -> Result<()> {
    let wasm_opt = match find_wasm_opt(cache, install_permitted)?.binary() {
        Ok(p) => p,
        Err(e) => {
            PBAR.info(&e.to_string());
            return Ok(());
        }
    };

    PBAR.info("Optimizing wasm binaries with `wasm-opt`...");

    for file in out_dir.read_dir()? {
        let file = file?;
        let path = file.path();
        if path.extension().and_then(|s| s.to_str()) != Some("wasm") {
            continue;
        }

        let tmp = path.with_extension("wasm-opt.wasm");
        let mut cmd = Command::new(&wasm_opt);
        cmd.arg(&path).arg("-o").arg(&tmp).args(args);
        child::run(cmd, "wasm-opt")?;
        std::fs::rename(&tmp, &path)?;
    }

    Ok(())
}

/// Attempts to find `wasm-opt` in `PATH` locally, or failing that downloads a
/// precompiled binary.
///
/// Returns `Some` if a binary was found or it was successfully downloaded.
/// Returns `None` if a binary wasn't found in `PATH` and this platform doesn't
/// have precompiled binaries. Returns an error if we failed to download the
/// binary.
pub fn find_wasm_opt(cache: &Cache, install_permitted: bool) -> Result<WasmOptLocation> {
    // First attempt to look up in PATH. If found assume it works.
    if let Ok(path) = which::which("wasm-opt") {
        PBAR.info(&format!("found wasm-opt at {:?}", path));
        return Ok(WasmOptLocation::Installed(path));
    }

    Ok(
        match install::download_prebuilt(
            &install::Tool::WasmOpt,
            cache,
            "latest",
            install_permitted,
        )? {
            install::Status::Found(download) => WasmOptLocation::Downloaded(download),
            install::Status::CannotInstall => WasmOptLocation::NoDownloadRequested,
            install::Status::PlatformNotSupported => WasmOptLocation::PlatformNotSupported,
        },
    )
}
