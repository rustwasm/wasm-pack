//! Getting WebDriver client binaries.

mod chromedriver;
mod geckodriver;
mod safaridriver;

use crate::PBAR;
use anyhow::Result;
use binary_install::Cache;
use std::path::PathBuf;

pub use self::{
    chromedriver::{get_or_install_chromedriver, install_chromedriver},
    geckodriver::{get_or_install_geckodriver, install_geckodriver},
    safaridriver::get_safaridriver,
};

// ------ driver helpers  ------

fn get_and_notify(
    cache: &Cache,
    installation_allowed: bool,
    name: &str,
    url: &str,
) -> Result<Option<PathBuf>> {
    if let Some(dl) = cache.download(false, name, &[name], url)? {
        return Ok(Some(dl.binary(name)?));
    }
    if installation_allowed {
        PBAR.info(&format!("Getting {}...", name));
    }
    match cache.download(installation_allowed, name, &[name], url)? {
        Some(dl) => Ok(Some(dl.binary(name)?)),
        None => Ok(None),
    }
}
