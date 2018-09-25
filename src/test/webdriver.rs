//! Getting WebDriver client binaries.

use binaries::{
    self, bin_path, install_binaries_from_targz_at_url, install_binaries_from_zip_at_url,
};
use command::build::BuildMode;
use error::Error;
use slog::Logger;
use std::path::{Path, PathBuf};
use target;

/// Get the path to an existing `chromedriver`, or install it if no existing
/// binary is found.
pub fn get_or_install_chromedriver(
    log: &Logger,
    crate_path: &Path,
    mode: BuildMode,
) -> Result<PathBuf, Error> {
    match (mode, bin_path(log, crate_path, "chromedriver")) {
        (_, Some(path)) => Ok(path),
        (BuildMode::Normal, None) => install_chromedriver(crate_path),
        (BuildMode::Force, None) => install_chromedriver(crate_path),
        (BuildMode::Noinstall, None) => Error::crate_config(
            "No crate-local `chromedriver` binary found, and could not find a global \
             `chromedriver` on the `$PATH`. Not installing `chromedriver` because of noinstall \
             mode.",
        ).map(|_| unreachable!()),
    }
}

fn get_local_chromedriver_path(crate_path: &Path) -> PathBuf {
    binaries::local_bin_path(crate_path, "chromedriver")
}

fn get_chromedriver_url() -> Result<String, Error> {
    let target = if target::LINUX && target::x86_64 {
        "linux64"
    } else if target::MACOS && target::x86_64 {
        "mac64"
    } else if target::WINDOWS && target::x86 {
        "win32"
    } else {
        return Err(Error::unsupported(
            "geckodriver binaries are unavailable for this target",
        ));
    };

    Ok(format!(
        "https://chromedriver.storage.googleapis.com/2.41/chromedriver_{}.zip",
        target
    ))
}

/// Download and install a pre-built `chromedriver` binary.
pub fn install_chromedriver(crate_path: &Path) -> Result<PathBuf, Error> {
    let url = get_chromedriver_url()?;
    install_binaries_from_zip_at_url(crate_path, &url, Some("chromedriver"))?;
    let chromedriver = get_local_chromedriver_path(crate_path);
    assert!(chromedriver.is_file());
    Ok(chromedriver)
}

/// Get the path to an existing `geckodriver`, or install it if no existing
/// binary is found.
pub fn get_or_install_geckodriver(
    log: &Logger,
    crate_path: &Path,
    mode: BuildMode,
) -> Result<PathBuf, Error> {
    match (mode, bin_path(log, crate_path, "geckodriver")) {
        (_, Some(path)) => Ok(path),
        (BuildMode::Normal, None) => install_geckodriver(crate_path),
        (BuildMode::Force, None) => install_geckodriver(crate_path),
        (BuildMode::Noinstall, None) => Error::crate_config(
            "No crate-local `geckodriver` binary found, and could not find a global `geckodriver` \
             on the `$PATH`. Not installing `geckodriver` because of noinstall mode.",
        ).map(|_| unreachable!()),
    }
}

fn get_geckodriver_url() -> Result<String, Error> {
    let (target, ext) = if target::LINUX && target::x86 {
        ("linux32", "tar.gz")
    } else if target::LINUX && target::x86_64 {
        ("linux64", "tar.gz")
    } else if target::MACOS {
        ("macos", "tar.gz")
    } else if target::WINDOWS && target::x86 {
        ("win32", "zip")
    } else if target::WINDOWS && target::x86_64 {
        ("win64", "zip")
    } else {
        return Err(Error::unsupported(
            "geckodriver binaries are unavailable for this target",
        ));
    };

    Ok(format!(
        "https://github.com/mozilla/geckodriver/releases/download/v0.21.0/geckodriver-v0.21.0-{}.{}",
        target,
        ext,
    ))
}

fn get_local_geckodriver_path(crate_path: &Path) -> PathBuf {
    binaries::local_bin_path(crate_path, "geckodriver")
}

/// Download and install a pre-built `geckodriver` binary.
pub fn install_geckodriver(crate_path: &Path) -> Result<PathBuf, Error> {
    let url = get_geckodriver_url()?;

    if url.ends_with("tar.gz") {
        install_binaries_from_targz_at_url(crate_path, &url, Some("geckodriver"))?;
    } else {
        assert!(url.ends_with("zip"));
        install_binaries_from_zip_at_url(crate_path, &url, Some("geckodriver"))?;
    }

    let geckodriver = get_local_geckodriver_path(crate_path);
    assert!(geckodriver.is_file());
    Ok(geckodriver)
}

/// Get the path to an existing `safaridriver`.
///
/// We can't install `safaridriver` if an existing one is not found because
/// Apple does not provide pre-built binaries. However, `safaridriver` *should*
/// be present by default.
pub fn get_safaridriver(log: &Logger, crate_path: &Path) -> Result<PathBuf, Error> {
    if let Some(p) = bin_path(log, crate_path, "safaridriver") {
        Ok(p)
    } else {
        Error::crate_config("could not find `safaridriver` on the `$PATH`").map(|_| unreachable!())
    }
}
