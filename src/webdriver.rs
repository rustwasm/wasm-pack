//! Getting WebDriver client binaries.

use binaries::{bin_path, install_binaries_from_targz_at_url, install_binaries_from_zip_at_url};
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
        (BuildMode::Noinstall, None) => {
            Error::crate_config("could not find `chromedriver` on the `$PATH`")
                .map(|_| unreachable!())
        }
    }
}

/// Download and install a pre-built `chromedriver` binary.
pub fn install_chromedriver(crate_path: &Path) -> Result<PathBuf, Error> {
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

    let url = format!(
        "https://chromedriver.storage.googleapis.com/2.41/chromedriver_{}.zip",
        target
    );
    install_binaries_from_zip_at_url(crate_path, &url, Some("chromedriver"))?;

    let chromedriver = crate_path.join("bin").join(if cfg!(target_os = "windows") {
        "chromedriver.exe"
    } else {
        "chromedriver"
    });
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
        (BuildMode::Noinstall, None) => {
            Error::crate_config("could not find `geckodriver` on the `$PATH`")
                .map(|_| unreachable!())
        }
    }
}

/// Download and install a pre-built `geckodriver` binary.
pub fn install_geckodriver(crate_path: &Path) -> Result<PathBuf, Error> {
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

    let url = format!(
        "https://github.com/mozilla/geckodriver/releases/download/v0.21.0/geckodriver-v0.21.0-{}.{}",
        target,
        ext,
    );

    if ext == "tar.gz" {
        install_binaries_from_targz_at_url(crate_path, &url, Some("geckodriver"))?;
    } else {
        assert_eq!(ext, "zip");
        install_binaries_from_zip_at_url(crate_path, &url, Some("geckodriver"))?;
    }

    let geckodriver = crate_path.join("bin").join(if cfg!(target_os = "windows") {
        "geckodriver.exe"
    } else {
        "geckodriver"
    });
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
