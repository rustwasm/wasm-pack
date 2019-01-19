//! Getting WebDriver client binaries.

use binary_install::Cache;
use command::build::BuildMode;
use failure;
use std::path::PathBuf;
use target;

/// Get the path to an existing `chromedriver`, or install it if no existing
/// binary is found.
pub fn get_or_install_chromedriver(
    cache: &Cache,
    mode: BuildMode,
) -> Result<PathBuf, failure::Error> {
    if let Ok(path) = which::which("chromedriver") {
        return Ok(path);
    }
    let installation_allowed = match mode {
        BuildMode::Noinstall => false,
        _ => true,
    };
    install_chromedriver(cache, installation_allowed)
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
    } else if target::WINDOWS && target::x86 {
        "win32"
    } else {
        bail!("geckodriver binaries are unavailable for this target")
    };

    let url = format!(
        "https://chromedriver.storage.googleapis.com/2.41/chromedriver_{}.zip",
        target
    );
    match cache.download(
        installation_allowed,
        "chromedriver",
        &["chromedriver"],
        &url,
    )? {
        Some(dl) => Ok(dl.binary("chromedriver")?),
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
    mode: BuildMode,
) -> Result<PathBuf, failure::Error> {
    if let Ok(path) = which::which("geckodriver") {
        return Ok(path);
    }
    let installation_allowed = match mode {
        BuildMode::Noinstall => false,
        _ => true,
    };
    install_geckodriver(cache, installation_allowed)
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
        "https://github.com/mozilla/geckodriver/releases/download/v0.21.0/geckodriver-v0.21.0-{}.{}",
        target,
        ext,
    );
    match cache.download(installation_allowed, "geckodriver", &["geckodriver"], &url)? {
        Some(dl) => Ok(dl.binary("geckodriver")?),
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
