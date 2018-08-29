//! Code related to error handling for wasm-pack
use curl;
use serde_json;
use std::borrow::Cow;
use std::io;
use toml;

/// Errors that can potentially occur in `wasm-pack`.
#[derive(Debug, Fail)]
pub enum Error {
    /// Maps any underlying I/O errors that are thrown to this variant
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    /// A JSON serialization or deserialization error.
    #[fail(display = "{}", _0)]
    SerdeJson(#[cause] serde_json::Error),

    /// A TOML serialization or deserialization error.
    #[fail(display = "{}", _0)]
    SerdeToml(#[cause] toml::de::Error),

    #[fail(display = "{}", _0)]
    /// A curl error.
    Curl(#[cause] curl::Error),

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

    #[fail(display = "{}", message)]
    /// An error related to an archive that we downloaded.
    Archive {
        /// Error message.
        message: String,
    },

    #[fail(display = "{}", message)]
    /// Error when some operation or feature is unsupported for the current
    /// target or environment.
    Unsupported {
        /// Error message.
        message: String,
    },

    #[fail(display = "{}", message)]
    /// Error related to some HTTP request.
    Http {
        /// Error message.
        message: String,
    },
}

impl Error {
    /// Construct a CLI error.
    pub fn cli(message: &str, stderr: Cow<str>) -> Result<(), Self> {
        Err(Error::Cli {
            message: message.to_string(),
            stderr: stderr.to_string(),
        })
    }

    /// Construct a crate configuration error.
    pub fn crate_config(message: &str) -> Result<(), Self> {
        Err(Error::CrateConfig {
            message: message.to_string(),
        })
    }

    /// Construct an archive error.
    pub fn archive(message: &str) -> Self {
        Error::Archive {
            message: message.to_string(),
        }
    }

    /// Construct an unsupported error.
    pub fn unsupported(message: &str) -> Self {
        Error::Unsupported {
            message: message.to_string(),
        }
    }

    /// Construct an http error.
    pub fn http(message: &str) -> Self {
        Error::Http {
            message: message.to_string(),
        }
    }

    /// Get a string description of this error's type.
    pub fn error_type(&self) -> String {
        match self {
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
            Error::Curl(_) => "There was an error making an HTTP request with curl. Details:\n\n",
            Error::Archive {..} => "There was an error related to an archive file. Details:\n\n",
            Error::Unsupported {..} => "There was an unsupported operation attempted. Details:\n\n",
            Error::Http {..} => "There wasn an HTTP error. Details:\n\n",
        }.to_string()
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<curl::Error> for Error {
    fn from(e: curl::Error) -> Self {
        Error::Curl(e)
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
