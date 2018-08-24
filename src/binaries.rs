//! Utilities for finding and installing binaries that we depend on.

use curl;
use error::Error;
use failure;
use flate2;
use slog::Logger;
use std::collections::HashSet;
use std::ffi;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tar;
use which::which;
use zip;

/// Get the local (at `$CRATE/bin/$BIN`; preferred) or global (on `$PATH`) path
/// for the given binary.
pub fn bin_path(log: &Logger, crate_path: &Path, bin: &str) -> Option<PathBuf> {
    assert!(!bin.ends_with(".exe"));
    debug!(log, "Searching for {} binary...", bin);

    // Return the path to the local binary, if it exists.
    let local_path = |crate_path: &Path| -> Option<PathBuf> {
        let mut p = crate_path.to_path_buf();
        p.push("bin");
        if cfg!(target_os = "windows") {
            let mut bin = bin.to_string();
            bin.push_str(".exe");
            p.push(bin);
        } else {
            p.push(bin);
        }

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
        }).or_else(|| {
            debug!(log, "Could not find {} binary.", bin);
            None
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

/// Download the `.tar.gz` file at the given URL and unpack the given binaries
/// from it into the given crate.
///
/// Upon success, every `$BIN` in `binaries` will be at `$CRATE/bin/$BIN`.
pub fn install_binaries_from_targz_at_url<'a, I>(
    crate_path: &Path,
    url: &str,
    binaries: I,
) -> Result<(), Error>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut binaries: HashSet<_> = binaries.into_iter().map(ffi::OsStr::new).collect();

    let tarball = curl(&url).map_err(|e| Error::http(&e.to_string()))?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(&tarball[..]));

    let bin = crate_path.join("bin");
    fs::create_dir_all(&bin)?;

    for entry in archive.entries()? {
        let mut entry = entry?;

        let dest = match entry.path()?.file_stem() {
            Some(f) if binaries.contains(f) => {
                binaries.remove(f);
                bin.join(entry.path()?.file_name().unwrap())
            }
            _ => continue,
        };

        entry.unpack(dest)?;
    }

    if binaries.is_empty() {
        Ok(())
    } else {
        Err(Error::archive(&format!(
            "the tarball at {} was missing expected executables: {}",
            url,
            binaries
                .into_iter()
                .map(|s| s.to_string_lossy())
                .collect::<Vec<_>>()
                .join(", "),
        )))
    }
}

/// Install binaries from within the given zip at the given URL.
///
/// Upon success, the binaries will be at the `$CRATE/bin/$BIN` path.
pub fn install_binaries_from_zip_at_url<'a, I>(
    crate_path: &Path,
    url: &str,
    binaries: I,
) -> Result<(), Error>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut binaries: HashSet<_> = binaries.into_iter().map(ffi::OsStr::new).collect();

    let data = curl(&url).map_err(|e| Error::http(&e.to_string()))?;
    let data = io::Cursor::new(data);
    let mut zip = zip::ZipArchive::new(data)?;

    let bin = crate_path.join("bin");
    fs::create_dir_all(&bin)?;

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).unwrap();
        let entry_path = entry.sanitized_name();
        match entry_path.file_stem() {
            Some(f) if binaries.contains(f) => {
                binaries.remove(f);
                let mut dest = bin_open_options()
                    .write(true)
                    .create_new(true)
                    .open(bin.join(entry_path.file_name().unwrap()))?;
                io::copy(&mut entry, &mut dest)?;
            }
            _ => continue,
        };
    }

    if binaries.is_empty() {
        Ok(())
    } else {
        Err(Error::archive(&format!(
            "the zip at {} was missing expected executables: {}",
            url,
            binaries
                .into_iter()
                .map(|s| s.to_string_lossy())
                .collect::<Vec<_>>()
                .join(", "),
        )))
    }
}

#[cfg(unix)]
fn bin_open_options() -> fs::OpenOptions {
    use std::os::unix::fs::OpenOptionsExt;

    let mut opts = fs::OpenOptions::new();
    opts.mode(0o755);
    opts
}

#[cfg(not(unix))]
fn bin_open_options() -> fs::OpenOptions {
    fs::OpenOptions::new()
}
