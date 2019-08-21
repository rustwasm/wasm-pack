use super::{get_and_notify, Collector};
use binary_install::Cache;
use failure;
use install::InstallMode;
use std::path::PathBuf;
use target;

/// Get the path to an existing `geckodriver`, or install it if no existing
/// binary is found or if there is a new binary version.
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

    let url = get_geckodriver_url(target, ext)?;

    match get_and_notify(cache, installation_allowed, "geckodriver", &url)? {
        Some(path) => Ok(path),
        None => bail!(
            "No cached `geckodriver` binary found, and could not find a global `geckodriver` \
             on the `$PATH`. Not installing `geckodriver` because of noinstall mode."
        ),
    }
}

/// Get `geckodriver` download URL.
///
/// It returns the latest one without checking the installed `Firefox` version
/// - it should be relatively safe because each `geckodriver` supports many `Firefox` versions:
/// https://firefox-source-docs.mozilla.org/testing/geckodriver/Support.html#supported-platforms
fn get_geckodriver_url(target: &str, ext: &str) -> Result<String, failure::Error> {
    // JSON example: `{"id":15227534,"tag_name":"v0.24.0","update_url":"/mozzila...`
    let latest_tag_json = fetch_latest_geckodriver_tag_json()?;
    let latest_tag = get_tag_name_from_json(&latest_tag_json)?;
    Ok(assemble_geckodriver_url(&latest_tag, target, ext))
}

// ------ `get_geckodriver_url` steps ------

fn fetch_latest_geckodriver_tag_json() -> Result<String, failure::Error> {
    let mut headers = curl::easy::List::new();
    headers.append("Accept: application/json")?;

    let mut handle = curl::easy::Easy2::new(Collector(Vec::new()));
    handle.url("https://github.com/mozilla/geckodriver/releases/latest")?;
    handle.http_headers(headers)?;
    // We will be redirected from the `latest` placeholder to the specific tag name.
    handle.follow_location(true)?;
    handle.perform()?;

    let contents = handle.get_ref();
    Ok(String::from_utf8_lossy(&contents.0).into_owned())
}

fn get_tag_name_from_json(json: &str) -> Result<String, failure::Error> {
    let json: serde_json::Value = serde_json::from_str(json)?;
    json.get("tag_name")
        .and_then(|tag_name| tag_name.as_str().map(ToOwned::to_owned))
        .ok_or_else(|| failure::err_msg("cannot get `tag_name` from JSON response"))
}

fn assemble_geckodriver_url(tag: &str, target: &str, ext: &str) -> String {
    format!(
        "https://github.com/mozilla/geckodriver/releases/download/{tag}/geckodriver-{tag}-{target}.{ext}",
        tag=tag,
        target=target,
        ext=ext,
    )
}
