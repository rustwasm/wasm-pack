//! Utilities for finding and installing binaries that we depend on.

extern crate curl;
#[macro_use]
extern crate failure;
extern crate flate2;

#[macro_use]
extern crate slog;
extern crate tar;
extern crate which;
extern crate zip;

pub mod error;
pub mod path;

mod fetch;
mod target;

use error::Error;
use fetch::curl;
use path::{ensure_local_bin_dir, local_bin_dir};
use std::collections::HashSet;
use std::ffi;
use std::fs;
use std::io;
use std::path::Path;

/// Download the `.tar.gz` file at the given URL and unpack the given binaries
/// from it into the given crate.
///
/// Upon success, every `$BIN` in `binaries` will be at `$CRATE/bin/$BIN`.
pub fn install_binaries_from_targz_at_url<'a, I>(
    crate_path: &Path,
    url: &str,
    binaries: I,
) -> Result<(), failure::Error>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut binaries: HashSet<_> = binaries.into_iter().map(ffi::OsStr::new).collect();

    let tarball = curl(&url).map_err(|e| Error::http(&e.to_string()))?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(&tarball[..]));

    ensure_local_bin_dir(crate_path)?;
    let bin = local_bin_dir(crate_path);

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
        ))
        .into())
    }
}

/// Install binaries from within the given zip at the given URL.
///
/// Upon success, the binaries will be at the `$CRATE/bin/$BIN` path.
pub fn install_binaries_from_zip_at_url<'a, I>(
    crate_path: &Path,
    url: &str,
    binaries: I,
) -> Result<(), failure::Error>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut binaries: HashSet<_> = binaries.into_iter().map(ffi::OsStr::new).collect();

    let data = curl(&url).map_err(|e| Error::http(&e.to_string()))?;
    let data = io::Cursor::new(data);
    let mut zip = zip::ZipArchive::new(data)?;

    ensure_local_bin_dir(crate_path)?;
    let bin = local_bin_dir(crate_path);

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
        ))
        .into())
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
