use std::ffi::{OsStr, OsString};

/// The `InstallMode` determines which mode of initialization we are running, and
/// what install steps we perform.
#[derive(Clone, Copy, Debug, Default)]
pub enum InstallMode {
    /// Perform all the install steps.
    #[default]
    Normal,
    /// Don't install tools like `wasm-bindgen`, just use the global
    /// environment's existing versions to do builds.
    Noinstall,
    /// Skip the rustc version check
    Force,
}

impl TryFrom<&OsStr> for InstallMode {
    type Error = OsString;

    fn try_from(s: &OsStr) -> Result<Self, OsString> {
        if s == "no-install" {
            Ok(InstallMode::Noinstall)
        } else if s == "normal" {
            Ok(InstallMode::Normal)
        } else if s == "force" {
            Ok(InstallMode::Force)
        } else {
            let mut err = OsString::from("Unknown build mode: ");
            err.push(s);
            Err(err)
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
