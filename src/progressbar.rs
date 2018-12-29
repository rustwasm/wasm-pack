//! Fancy progress bar functionality.

use console::style;
use emoji;
use indicatif::{ProgressBar, ProgressStyle};
use parking_lot::RwLock;
use std::fmt;

/// Synchronized progress bar and status message printing.
pub struct ProgressOutput {
    spinner: RwLock<ProgressBar>,
    messages: RwLock<String>,
}

impl ProgressOutput {
    /// Construct a new `ProgressOutput`.
    pub fn new() -> Self {
        Self {
            spinner: RwLock::new(ProgressBar::new_spinner()),
            messages: RwLock::new(String::from("")),
        }
    }

    /// Inform the user that the given `step` is being executed, with details in
    /// `message`.
    pub fn step(&self, step: &Step, message: &str) {
        let msg = format!("{} {}", style(step).bold().dim(), message);
        self.message(&msg)
    }

    fn finish(&self) {
        let spinner = self.spinner.read();
        spinner.finish();

        let mut message = self.messages.write();
        print!("{}", *message);
        message.clear();
    }

    /// Print the given message.
    pub fn message(&self, message: &str) {
        self.finish();

        let mut spinner = self.spinner.write();
        *spinner = Self::progressbar(message);
    }

    fn add_message(&self, msg: &str) {
        let mut message = self.messages.write();
        message.push_str("  ");
        message.push_str(msg);
        message.push('\n');
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

    fn progressbar(msg: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(200);
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("/|\\- ")
                .template("{spinner:.dim.bold} {wide_msg}"),
        );
        pb.set_message(&msg);
        pb
    }

    /// After having built up a series of messages, print all of them out.
    pub fn done(&self) {
        self.finish();
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
        write!(f, "[{}/{}]", self.current, self.total)
    }
}

impl Drop for ProgressOutput {
    fn drop(&mut self) {
        self.done();
    }
}

impl Default for ProgressOutput {
    fn default() -> Self {
        Self::new()
    }
}
