//! Fancy progress bar functionality.

use console::style;
use emoji;
use std::fmt;

/// Synchronized progress bar and status message printing.
pub struct ProgressOutput;

impl ProgressOutput {
    /// Inform the user that the given `step` is being executed, with details in
    /// `message`.
    pub fn step(&self, step: &Step, message: &str) {
        let msg = format!("{} {}", style(step).bold().dim(), message);
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

/// For processes that can be broken down into N fractional steps, with messages
/// added for each step along the way like
///
/// > [2/5] Doing the second step out of five.
pub struct Step {
    current: usize,
    total: usize,
}

impl Step {
    /// Construct a `Step` where there are `total` number of steps.
    pub fn new(total: usize) -> Step {
        Step { current: 1, total }
    }

    /// Increment the current step.
    pub fn inc(&mut self) {
        self.current += 1;
    }
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", emoji::INFO)
    }
}

impl Default for ProgressOutput {
    fn default() -> Self {
        ProgressOutput
    }
}
