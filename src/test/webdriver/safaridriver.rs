use anyhow::{bail, Result};
use std::path::PathBuf;

/// Get the path to an existing `safaridriver`.
///
/// We can't install `safaridriver` if an existing one is not found because
/// Apple does not provide pre-built binaries. However, `safaridriver` *should*
/// be present by default.
pub fn get_safaridriver() -> Result<PathBuf> {
    match which::which("safaridriver") {
        Ok(p) => Ok(p),
        Err(_) => bail!("could not find `safaridriver` on the `$PATH`"),
    }
}
