//! Fancy progress bar functionality.

use console::style;
use emoji;

/// Synchronized progress bar and status message printing.
pub struct ProgressOutput;

impl ProgressOutput {
    /// Inform the user that the given `step` is being executed, with details in
    /// `message`.
    pub fn step(&self, message: &str) {
        self.info(message);
    }

    /// Print the given message.
    pub fn message(&self, message: &str) {
        eprintln!("{}", message);
    }

    /// Add an informational message.
    pub fn info(&self, message: &str) {
        let info = format!(
            "{} {}: {}",
            emoji::INFO,
            style("[INFO]").bold().dim(),
            message
        );
        self.message(&info);
    }

    /// Add a warning message.
    pub fn warn(&self, message: &str) {
        let warn = format!(
            "{} {}: {}",
            emoji::WARN,
            style("[WARN]").bold().dim(),
            message
        );
        self.message(&warn);
    }

    /// Add an error message.
    pub fn error(&self, message: &str) {
        let err = format!(
            "{} {}: {}",
            emoji::ERROR,
            style("[ERR]").bold().dim(),
            message
        );
        self.message(&err);
    }
}

impl Default for ProgressOutput {
    fn default() -> Self {
        ProgressOutput
    }
}
