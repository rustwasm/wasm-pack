//! Fancy progress bar functionality.

use console::style;
use emoji;

/// Synchronized progress bar and status message printing.
pub struct ProgressOutput;

impl ProgressOutput {
    /// Inform the user that the given `step` is being executed, with details in
    /// `message`.
    pub fn step(&self, message: &str) {
        let msg = format!("{} {}", style(emoji::INFO).bold().dim(), message);
        self.message(&msg)
    }

    /// Print the given message.
    pub fn message(&self, message: &str) {
        eprintln!("  {}", message);
    }

    fn add_message(&self, msg: &str) {
        println!("{}", msg);
    }

    /// Add an informational message.
    pub fn info(&self, message: &str) {
        let info = format!(
            "{} {}: {}",
            emoji::INFO,
            style("[INFO]").bold().dim(),
            message
        );
        self.add_message(&info);
    }

    /// Add a warning message.
    pub fn warn(&self, message: &str) {
        let warn = format!(
            "{} {}: {}",
            emoji::WARN,
            style("[WARN]").bold().dim(),
            message
        );
        self.add_message(&warn);
    }

    /// Add an error message.
    pub fn error(&self, message: &str) {
        let err = format!(
            "{} {}: {}",
            emoji::ERROR,
            style("[ERR]").bold().dim(),
            message
        );
        self.add_message(&err);
    }
}

impl Default for ProgressOutput {
    fn default() -> Self {
        ProgressOutput
    }
}
