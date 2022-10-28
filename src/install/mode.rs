use anyhow::{bail, Error, Result};
use std::str::FromStr;

/// The `InstallMode` determines which mode of initialization we are running, and
/// what install steps we perform.
#[derive(Clone, Copy, Debug)]
pub enum InstallMode {
    /// Perform all the install steps.
    Normal,
    /// Don't install tools like `wasm-bindgen`, just use the global
    /// environment's existing versions to do builds.
    Noinstall,
    /// Skip the rustc version check
    Force,
}

impl Default for InstallMode {
    fn default() -> InstallMode {
        InstallMode::Normal
    }
}

impl FromStr for InstallMode {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "no-install" => Ok(InstallMode::Noinstall),
            "normal" => Ok(InstallMode::Normal),
            "force" => Ok(InstallMode::Force),
            _ => bail!("Unknown build mode: {}", s),
        }
    }
}

impl InstallMode {
    /// Determines if installation is permitted during a function call based on --mode flag
    pub fn install_permitted(self) -> bool {
        match self {
            InstallMode::Normal => true,
            InstallMode::Force => true,
            InstallMode::Noinstall => false,
        }
    }
}
