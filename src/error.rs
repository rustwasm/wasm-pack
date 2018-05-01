//! Code related to error handling for wasm-pack
use serde_json;
use std::borrow::Cow;
use std::io;
use toml;

#[derive(Debug, Fail)]
pub enum Error {
    /// Maps any underlying I/O errors that are thrown to this variant
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
    #[fail(display = "{}", _0)]
    SerdeJson(#[cause] serde_json::Error),
    #[fail(display = "{}", _0)]
    SerdeToml(#[cause] toml::de::Error),
    #[fail(display = "{}. stderr:\n\n{}", message, stderr)]
    Cli { message: String, stderr: String },
}

impl Error {
    pub fn cli(message: &str, stderr: Cow<str>) -> Result<(), Self> {
        Err(Error::Cli {
            message: message.to_string(),
            stderr: stderr.to_string(),
        })
    }

    pub fn error_type(&self) -> String {
        match self {
            Error::Io(_) => "There was an I/O error. Details:\n\n",
            Error::SerdeJson(_) => "There was an JSON error. Details:\n\n",
            Error::SerdeToml(_) => "There was an TOML error. Details:\n\n",
            Error::Cli {
                message: _,
                stderr: _,
            } => "There was an error while calling another CLI tool. Details:\n\n",
        }.to_string()
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
