//! Utilities for finding and installing binaries that we depend on.

extern crate curl;
#[macro_use]
extern crate failure;
extern crate dirs;
extern crate flate2;
extern crate hex;
extern crate is_executable;
extern crate siphasher;
extern crate tar;
extern crate zip;

use failure::{Error, ResultExt};
use siphasher::sip::SipHasher13;
use std::collections::HashSet;
use std::env;
use std::ffi;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};

/// Global cache for wasm-pack, currently containing binaries downloaded from
/// urls like wasm-bindgen and such.
#[derive(Debug)]
pub struct Cache {
    destination: PathBuf,
}

/// Representation of a downloaded tarball/zip
#[derive(Debug)]
pub struct Download {
    root: PathBuf,
}

impl Cache {
    /// Returns the global cache directory, as inferred from env vars and such.
    ///
    /// This function may return an error if a cache directory cannot be
    /// determined.
    pub fn new(name: &str) -> Result<Cache, Error> {
        let cache_name = format!(".{}", name);
        let destination = dirs::cache_dir()
            .map(|p| p.join(&cache_name))
            .or_else(|| {
                let home = dirs::home_dir()?;
                Some(home.join(&cache_name))
            })
            .ok_or_else(|| format_err!("couldn't find your home directory, is $HOME not set?"))?;
        Ok(Cache::at(&destination))
    }

    /// Creates a new cache specifically at a particular directory, useful in
    /// testing and such.
    pub fn at(path: &Path) -> Cache {
        Cache {
            destination: path.to_path_buf(),
        }
    }

    /// Joins a path to the destination of this cache, returning the result
    pub fn join(&self, path: &Path) -> PathBuf {
        self.destination.join(path)
    }

    /// Downloads a tarball or zip file from the specified url, extracting it
    /// locally and returning the directory that the contents were extracted
    /// into.
    ///
    /// Note that this function requries that the contents of `url` never change
    /// as the contents of the url are globally cached on the system and never
    /// invalidated.
    ///
    /// The `name` is a human-readable name used to go into the folder name of
    /// the destination, and `binaries` is a list of binaries expected to be at
    /// the url. If the URL's extraction doesn't contain all the binaries this
    /// function will return an error.
    pub fn download(
        &self,
        install_permitted: bool,
        name: &str,
        binaries: &[&str],
        url: &str,
    ) -> Result<Option<Download>, Error> {
        let dirname = hashed_dirname(url, name);

        let destination = self.destination.join(&dirname);

        if destination.exists() {
            return Ok(Some(Download { root: destination }));
        }

        if !install_permitted {
            return Ok(None);
        }

        let data = curl(&url).with_context(|_| format!("failed to download from {}", url))?;

        // Extract everything in a temporary directory in case we're ctrl-c'd.
        // Don't want to leave around corrupted data!
        let temp = self.destination.join(&format!(".{}", dirname));
        drop(fs::remove_dir_all(&temp));
        fs::create_dir_all(&temp)?;

        if url.ends_with(".tar.gz") {
            self.extract_tarball(&data, &temp, binaries)
                .with_context(|_| format!("failed to extract tarball from {}", url))?;
        } else if url.ends_with(".zip") {
            self.extract_zip(&data, &temp, binaries)
                .with_context(|_| format!("failed to extract zip from {}", url))?;
        } else {
            // panic instead of runtime error as it's a static violation to
            // download a different kind of url, all urls should be encoded into
            // the binary anyway
            panic!("don't know how to extract {}", url)
        }

        // Now that everything is ready move this over to our destination and
        // we're good to go.
        fs::rename(&temp, &destination)?;
        Ok(Some(Download { root: destination }))
    }

    fn extract_tarball(&self, tarball: &[u8], dst: &Path, binaries: &[&str]) -> Result<(), Error> {
        let mut binaries: HashSet<_> = binaries.into_iter().map(ffi::OsStr::new).collect();
        let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(tarball));

        for entry in archive.entries()? {
            let mut entry = entry?;

            let dest = match entry.path()?.file_stem() {
                Some(f) if binaries.contains(f) => {
                    binaries.remove(f);
                    dst.join(entry.path()?.file_name().unwrap())
                }
                _ => continue,
            };

            entry.unpack(dest)?;
        }

        if !binaries.is_empty() {
            bail!(
                "the tarball was missing expected executables: {}",
                binaries
                    .into_iter()
                    .map(|s| s.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        }

        Ok(())
    }

    fn extract_zip(&self, zip: &[u8], dst: &Path, binaries: &[&str]) -> Result<(), Error> {
        let mut binaries: HashSet<_> = binaries.into_iter().map(ffi::OsStr::new).collect();

        let data = io::Cursor::new(zip);
        let mut zip = zip::ZipArchive::new(data)?;

        for i in 0..zip.len() {
            let mut entry = zip.by_index(i).unwrap();
            let entry_path = entry.sanitized_name();
            match entry_path.file_stem() {
                Some(f) if binaries.contains(f) => {
                    binaries.remove(f);
                    let mut dest = bin_open_options()
                        .write(true)
                        .create_new(true)
                        .open(dst.join(entry_path.file_name().unwrap()))?;
                    io::copy(&mut entry, &mut dest)?;
                }
                _ => continue,
            };
        }

        if !binaries.is_empty() {
            bail!(
                "the zip was missing expected executables: {}",
                binaries
                    .into_iter()
                    .map(|s| s.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        }

        return Ok(());

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
    }
}

impl Download {
    /// Manually constructs a download at the specified path
    pub fn at(path: &Path) -> Download {
        Download {
            root: path.to_path_buf(),
        }
    }

    /// Returns the path to the binary `name` within this download
    pub fn binary(&self, name: &str) -> Result<PathBuf, Error> {
        use is_executable::IsExecutable;

        let ret = self
            .root
            .join(name)
            .with_extension(env::consts::EXE_EXTENSION);

        if !ret.is_file() {
            bail!("{} binary does not exist", ret.display());
        }
        if !ret.is_executable() {
            bail!("{} is not executable", ret.display());
        }

        Ok(ret)
    }
}

fn curl(url: &str) -> Result<Vec<u8>, Error> {
    let mut data = Vec::new();

    let mut easy = curl::easy::Easy::new();
    easy.follow_location(true)?;
    easy.url(url)?;
    easy.get(true)?;
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|part| {
            data.extend_from_slice(part);
            Ok(part.len())
        })?;
        transfer.perform()?;
    }

    let status_code = easy.response_code()?;
    if 200 <= status_code && status_code < 300 {
        Ok(data)
    } else {
        bail!(
            "received a bad HTTP status code ({}) when requesting {}",
            status_code,
            url
        )
    }
}

fn hashed_dirname(url: &str, name: &str) -> String {
    let mut hasher = SipHasher13::new();
    url.hash(&mut hasher);
    let result = hasher.finish();
    let hex = hex::encode(&[
        (result >> 0) as u8,
        (result >> 8) as u8,
        (result >> 16) as u8,
        (result >> 24) as u8,
        (result >> 32) as u8,
        (result >> 40) as u8,
        (result >> 48) as u8,
        (result >> 56) as u8,
    ]);
    format!("{}-{}", name, hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_same_hash_for_same_name_and_url() {
        let name = "wasm-pack";
        let url = "http://localhost:7878/wasm-pack-v0.6.0.tar.gz";

        let first = hashed_dirname(url, name);
        let second = hashed_dirname(url, name);

        assert!(!first.is_empty());
        assert!(!second.is_empty());
        assert_eq!(first, second);
    }

    #[test]
    fn it_returns_different_hashes_for_different_urls() {
        let name = "wasm-pack";
        let url = "http://localhost:7878/wasm-pack-v0.5.1.tar.gz";
        let second_url = "http://localhost:7878/wasm-pack-v0.6.0.tar.gz";

        let first = hashed_dirname(url, name);
        let second = hashed_dirname(second_url, name);

        assert_ne!(first, second);
    }

    #[test]
    fn it_returns_cache_dir() {
        let name = "wasm-pack";
        let cache = Cache::new(name);

        let expected = dirs::cache_dir()
            .unwrap()
            .join(PathBuf::from(".".to_owned() + name));

        assert!(cache.is_ok());
        assert_eq!(cache.unwrap().destination, expected);
    }

    #[test]
    fn it_returns_destination_if_binary_already_exists() {
        use std::fs;

        let binary_name = "wasm-pack";
        let binaries = vec![binary_name];

        let dir = tempfile::TempDir::new().unwrap();
        let cache = Cache::at(dir.path());
        let url = &format!("{}/{}.tar.gz", "http://localhost:7878", binary_name);

        let dirname = hashed_dirname(&url, &binary_name);
        let full_path = dir.path().join(dirname);

        // Create temporary directory and binary to simulate that
        // a cached binary already exists.
        fs::create_dir_all(full_path).unwrap();

        let dl = cache.download(true, binary_name, &binaries, url);

        assert!(dl.is_ok());
        assert!(dl.unwrap().is_some())
    }
}
