//! Fancy progress bar functionality.

use console::style;
use emoji;
use std::sync::atomic::{AtomicBool, Ordering};

/// Synchronized progress bar and status message printing.
pub struct ProgressOutput {
    quiet: AtomicBool,
}

impl ProgressOutput {
    /// Returns a new ProgressOutput
    pub const fn new() -> Self {
        Self {
            quiet: AtomicBool::new(false),
        }
    }

    /// Print the given message.
    fn message(&self, message: &str) {
        eprintln!("{}", message);
    }

    fn quiet(&self) -> bool {
        self.quiet.load(Ordering::SeqCst)
    }

    /// Sets whether it should silence warnings or not
    pub fn set_quiet(&self, quiet: bool) {
        self.quiet.store(quiet, Ordering::SeqCst);
    }

    /// Add an informational message.
    pub fn info(&self, message: &str) {
        if !self.quiet() {
            let info = format!("{}: {}", style("[INFO]").bold().dim(), message,);
            self.message(&info);
        }
    }

    /// Add a warning message.
    pub fn warn(&self, message: &str) {
        if !self.quiet() {
            let warn = format!(
                "{} {}: {}",
                emoji::WARN,
                style("[WARN]").bold().dim(),
                message
            );
            self.message(&warn);
        }
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
        ProgressOutput::new()
    }
}
