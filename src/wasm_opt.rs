//! Support for downloading and executing `wasm-opt`

use crate::child;
use crate::emoji;
use crate::target;
use crate::PBAR;
use binary_install::Cache;
use log::debug;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Execute `wasm-opt` over wasm binaries found in `out_dir`, downloading if
/// necessary into `cache`. Passes `args` to each invocation of `wasm-opt`.
pub fn run(
    cache: &Cache,
    out_dir: &Path,
    args: &[String],
    install_permitted: bool,
) -> Result<(), failure::Error> {
    let wasm_opt = match find_wasm_opt(cache, install_permitted)? {
        WasmOpt::Found(path) => path,
        WasmOpt::CannotInstall => {
            PBAR.info("Skipping wasm-opt as no downloading was requested");
            return Ok(());
        }
        WasmOpt::PlatformNotSupported => {
            PBAR.info("Skipping wasm-opt because it is not supported on this platform");
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

/// Possible results of `find_wasm_opt`
pub enum WasmOpt {
    /// Couldn't install wasm-opt because downloads are forbidden
    CannotInstall,
    /// The current platform doesn't support precompiled binaries
    PlatformNotSupported,
    /// We found `wasm-opt` at the specified path
    Found(PathBuf),
}

/// Attempts to find `wasm-opt` in `PATH` locally, or failing that downloads a
/// precompiled binary.
///
/// Returns `Some` if a binary was found or it was successfully downloaded.
/// Returns `None` if a binary wasn't found in `PATH` and this platform doesn't
/// have precompiled binaries. Returns an error if we failed to download the
/// binary.
pub fn find_wasm_opt(cache: &Cache, install_permitted: bool) -> Result<WasmOpt, failure::Error> {
    // First attempt to look up in PATH. If found assume it works.
    if let Ok(path) = which::which("wasm-opt") {
        debug!("found wasm-opt at {:?}", path);
        return Ok(WasmOpt::Found(path));
    }

    // ... and if that fails download a precompiled version.
    let target = if target::LINUX && target::x86_64 {
        "x86_64-linux"
    } else if target::MACOS && target::x86_64 {
        "x86_64-apple-darwin"
    } else if target::WINDOWS && target::x86_64 {
        "x86_64-windows"
    } else {
        return Ok(WasmOpt::PlatformNotSupported);
    };
    let url = format!(
        "https://github.com/WebAssembly/binaryen/releases/download/{vers}/binaryen-{vers}-{target}.tar.gz",
        vers = "version_78",
        target = target,
    );

    let download = |permit_install| cache.download(permit_install, "wasm-opt", &["wasm-opt"], &url);

    let dl = match download(false)? {
        Some(dl) => dl,
        None if !install_permitted => return Ok(WasmOpt::CannotInstall),
        None => {
            let msg = format!("{}Installing wasm-opt...", emoji::DOWN_ARROW);
            PBAR.info(&msg);

            match download(install_permitted)? {
                Some(dl) => dl,
                None => return Ok(WasmOpt::CannotInstall),
            }
        }
    };

    Ok(WasmOpt::Found(dl.binary("wasm-opt")?))
}
