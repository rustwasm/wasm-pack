//! Code related to error handling for binary-install

/// Errors that can potentially occur in `binary-install`.
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", message)]
    /// An error related to an archive that we downloaded.
    Archive {
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
    /// Construct an archive error.
    pub fn archive(message: &str) -> Self {
        Error::Archive {
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
            Error::Archive { .. } => "There was an error related to an archive file. Details:\n\n",
            Error::Http { .. } => "There wasn an HTTP error. Details:\n\n",
        }
        .to_string()
    }
}
