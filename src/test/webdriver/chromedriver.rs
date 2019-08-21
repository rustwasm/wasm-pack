use super::{get_and_notify, Collector};
use binary_install::Cache;
use failure;
use install::InstallMode;
use std::path::PathBuf;
use target;

/// Get the path to an existing `chromedriver`, or install it if no existing
/// binary is found or if there is a new binary version.
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

    let url = get_chromedriver_url(target)?;

    match get_and_notify(cache, installation_allowed, "chromedriver", &url)? {
        Some(path) => Ok(path),
        None => bail!(
            "No cached `chromedriver` binary found, and could not find a global \
             `chromedriver` on the `$PATH`. Not installing `chromedriver` because of noinstall \
             mode."
        ),
    }
}

/// Get `chromedriver` download URL.
///
/// It returns the latest one without checking the installed `Chrome` version
/// because it's not easy to find out `Chrome` version on `Windows` -
/// https://bugs.chromium.org/p/chromium/issues/detail?id=158372
///
/// The official algorithm for `chromedriver` version selection:
/// https://chromedriver.chromium.org/downloads/version-selection
fn get_chromedriver_url(target: &str) -> Result<String, failure::Error> {
    let chromedriver_version = fetch_chromedriver_version()?;
    Ok(assemble_chromedriver_url(&chromedriver_version, target))
}

// ------ `get_chromedriver_url` steps ------

fn fetch_chromedriver_version() -> Result<String, failure::Error> {
    let mut handle = curl::easy::Easy2::new(Collector(Vec::new()));
    handle.url("https://chromedriver.storage.googleapis.com/LATEST_RELEASE")?;
    handle.perform()?;

    let contents = handle.get_ref();
    Ok(String::from_utf8_lossy(&contents.0).into_owned())
}

fn assemble_chromedriver_url(chromedriver_version: &str, target: &str) -> String {
    format!(
        "https://chromedriver.storage.googleapis.com/{version}/chromedriver_{target}.zip",
        version = chromedriver_version,
        target = target,
    )
}
