//! Getting WebDriver client binaries.

mod chromedriver;
mod geckodriver;
mod safaridriver;

use binary_install::Cache;
use failure;
use std::path::PathBuf;
use PBAR;

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
) -> Result<Option<PathBuf>, failure::Error> {
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

struct Collector(Vec<u8>);

impl Collector {
    pub fn take_content(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.0)
    }
}

impl curl::easy::Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, curl::easy::WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}
