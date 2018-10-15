use slog::Logger;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use target;
use which::which;

/// Get the path for a crate's directory of locally-installed binaries.
///
/// This does not check whether or ensure that the directory exists.
pub fn local_bin_dir(crate_path: &Path) -> PathBuf {
    crate_path.join("bin")
}

/// Ensure that the crate's directory for locally-installed binaries exists.
pub fn ensure_local_bin_dir(crate_path: &Path) -> io::Result<()> {
    fs::create_dir_all(local_bin_dir(crate_path))
}

/// Get the path for where `bin` would be if we have a crate-local install for
/// it.
///
/// This does *not* check whether there is a file at that path or not.
///
/// This will automatically add the `.exe` extension for windows.
pub fn local_bin_path(crate_path: &Path, bin: &str) -> PathBuf {
    let mut p = local_bin_dir(crate_path).join(bin);
    if target::WINDOWS {
        p.set_extension("exe");
    }
    p
}

/// Get the local (at `$CRATE/bin/$BIN`; preferred) or global (on `$PATH`) path
/// for the given binary.
///
/// If this function returns `Some(path)`, then a file at that path exists (or
/// at least existed when we checked! In general, we aren't really worried about
/// racing with an uninstall of a tool that we rely on.)
pub fn bin_path(log: &Logger, crate_path: &Path, bin: &str) -> Option<PathBuf> {
    assert!(!bin.ends_with(".exe"));
    debug!(log, "Searching for {} binary...", bin);

    // Return the path to the local binary, if it exists.
    let local_path = |crate_path: &Path| -> Option<PathBuf> {
        let p = local_bin_path(crate_path, bin);
        debug!(log, "Checking for local {} binary at {}", bin, p.display());
        if p.is_file() {
            Some(p)
        } else {
            None
        }
    };

    // Return the path to the global binary, if it exists.
    let global_path = || -> Option<PathBuf> {
        debug!(log, "Looking for global {} binary on $PATH", bin);
        if let Ok(p) = which(bin) {
            Some(p)
        } else {
            None
        }
    };

    local_path(crate_path)
        .or_else(global_path)
        .map(|p| {
            let p = p.canonicalize().unwrap_or(p);
            debug!(log, "Using {} binary at {}", bin, p.display());
            p
        })
        .or_else(|| {
            debug!(log, "Could not find {} binary.", bin);
            None
        })
}
