use anyhow::{bail, Result};

use crate::target;

/// An enum representing supported operating systems
#[derive(Clone, PartialEq, Eq)]
pub enum Os {
    /// Linux operating system
    Linux,
    /// Macos operating system
    MacOS,
    /// Windows operating system
    Windows,
}

impl Os {
    /// Get the current operating system
    pub fn get() -> Result<Self> {
        if target::LINUX {
            Ok(Os::Linux)
        } else if target::MACOS {
            Ok(Os::MacOS)
        } else if target::WINDOWS {
            Ok(Os::Windows)
        } else {
            bail!("Unrecognized target!")
        }
    }

    /// Returns the OS name
    pub fn name(&self) -> &'static str {
        match self {
            Os::Linux => "linux",
            Os::MacOS => "macOS",
            Os::Windows => "windows",
        }
    }
}
