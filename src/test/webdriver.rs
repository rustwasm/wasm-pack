//! Getting WebDriver client binaries.

use binary_install::Cache;
use failure;
use install::InstallMode;
use std::path::PathBuf;
use target;
use PBAR;

fn get_and_notify(
    cache: &Cache,
    installation_allowed: bool,
    name: &str,
    url: &str,
) -> Result<Option<PathBuf>, failure::Error> {
    if let Some(dl) = cache.download(false, name, &[name], &url)? {
        return Ok(Some(dl.binary(name)?));
    }
    if installation_allowed {
        PBAR.info(&format!("Getting {}...", name));
    }
    match cache.download(installation_allowed, name, &[name], &url)? {
        Some(dl) => Ok(Some(dl.binary(name)?)),
        None => Ok(None),
    }
}

/// Get the path to an existing `chromedriver`, or install it if no existing
/// binary is found.
pub fn get_or_install_chromedriver(
    cache: &Cache,
    mode: InstallMode,
) -> Result<PathBuf, failure::Error> {
    if let Ok(path) = which::which("chromedriver") {
        return Ok(path);
    }
    install_chromedriver(cache, mode.install_permitted())
}

/// Download and install a pre-built `chromedriver` binary.
pub fn install_chromedriver(
    cache: &Cache,
    installation_allowed: bool,
) -> Result<PathBuf, failure::Error> {
    let target = if target::LINUX && target::x86_64 {
        "linux64"
    } else if target::MACOS && target::x86_64 {
        "mac64"
    } else if target::WINDOWS {
        "win32"
    } else {
        bail!("chromedriver binaries are unavailable for this target")
    };

    let url = format!(
        "https://chromedriver.storage.googleapis.com/2.46/chromedriver_{}.zip",
        target
    );
    match get_and_notify(cache, installation_allowed, "chromedriver", &url)? {
        Some(path) => Ok(path),
        None => bail!(
            "No cached `chromedriver` binary found, and could not find a global \
             `chromedriver` on the `$PATH`. Not installing `chromedriver` because of noinstall \
             mode."
        ),
    }
}

/// Get the path to an existing `geckodriver`, or install it if no existing
/// binary is found.
pub fn get_or_install_geckodriver(
    cache: &Cache,
    mode: InstallMode,
) -> Result<PathBuf, failure::Error> {
    if let Ok(path) = which::which("geckodriver") {
        return Ok(path);
    }
    install_geckodriver(cache, mode.install_permitted())
}

/// Download and install a pre-built `geckodriver` binary.
pub fn install_geckodriver(
    cache: &Cache,
    installation_allowed: bool,
) -> Result<PathBuf, failure::Error> {
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
        bail!("geckodriver binaries are unavailable for this target")
    };

    let url = format!(
        "https://github.com/mozilla/geckodriver/releases/download/v0.24.0/geckodriver-v0.24.0-{}.{}",
        target,
        ext,
    );
    match get_and_notify(cache, installation_allowed, "geckodriver", &url)? {
        Some(path) => Ok(path),
        None => bail!(
            "No cached `geckodriver` binary found, and could not find a global `geckodriver` \
             on the `$PATH`. Not installing `geckodriver` because of noinstall mode."
        ),
    }
}

/// Get the path to an existing `safaridriver`.
///
/// We can't install `safaridriver` if an existing one is not found because
/// Apple does not provide pre-built binaries. However, `safaridriver` *should*
/// be present by default.
pub fn get_safaridriver() -> Result<PathBuf, failure::Error> {
    match which::which("safaridriver") {
        Ok(p) => Ok(p),
        Err(_) => bail!("could not find `safaridriver` on the `$PATH`"),
    }
}
