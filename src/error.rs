//! Code related to error handling for wasm-pack
use serde_json;
use std::borrow::Cow;
use std::io;
use std::env;
use toml;

/// Errors that can potentially occur in `wasm-pack`.
#[derive(Debug, Fail)]
pub enum Error {
    /// Maps any underlying environment errors that are thrown to this variant.
    #[fail(display = "{}", _0)]
    Env(#[cause] env::VarError),

    /// Maps any underlying I/O errors that are thrown to this variant
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    /// A JSON serialization or deserialization error.
    #[fail(display = "{}", _0)]
    SerdeJson(#[cause] serde_json::Error),

    /// A TOML serialization or deserialization error.
    #[fail(display = "{}", _0)]
    SerdeToml(#[cause] toml::de::Error),

    /// An error invoking another CLI tool.
    #[fail(display = "{}. stderr:\n\n{}", message, stderr)]
    Cli {
        /// Error message.
        message: String,
        /// The underlying CLI's `stderr` output.
        stderr: String,
    },

    /// A crate configuration error.
    #[fail(display = "{}", message)]
    CrateConfig {
        /// A message describing the configuration error.
        message: String,
    },
    #[fail(display = "{}", message)]
    /// Error when the 'pkg' directory is not found.
    PkgNotFound {
        /// Message describing the error.
        message: String,
    },
}

impl Error {
    /// Construct a CLI error.
    pub fn cli(message: &str, stderr: Cow<str>) -> Self {
        Error::Cli {
            message: message.to_string(),
            stderr: stderr.to_string(),
        }
    }

    /// Construct a crate configuration error.
    pub fn crate_config(message: &str) -> Self {
        Error::CrateConfig {
            message: message.to_string(),
        }
    }

    /// Get a string description of this error's type.
    pub fn error_type(&self) -> String {
        match self {
            Error::Env(_) => "There was an environment error. Details:\n\n",
            Error::Io(_) => "There was an I/O error. Details:\n\n",
            Error::SerdeJson(_) => "There was an JSON error. Details:\n\n",
            Error::SerdeToml(_) => "There was an TOML error. Details:\n\n",
            Error::Cli {
                message: _,
                stderr: _,
            } => "There was an error while calling another CLI tool. Details:\n\n",
            Error::CrateConfig { message: _ } => {
                "There was a crate configuration error. Details:\n\n"
            }
            Error::PkgNotFound {
                message: _,
            } => "Unable to find the 'pkg' directory at the path, set the path as the parent of the 'pkg' directory \n\n",
        }.to_string()
    }
}

impl From<env::VarError> for Error {
    fn from(e: env::VarError) -> Self {
        Error::Env(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerdeJson(e)
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::SerdeToml(e)
    }
}
