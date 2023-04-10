//! Fancy progress bar functionality.

use crate::emoji;
use anyhow::Result;
use console::style;
use std::ffi::{OsStr, OsString};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
/// The maximum log level for wasm-pack
// The ordering is important: the least verbose must be at
// the top and the most verbose at the bottom
pub enum LogLevel {
    /// Logs only error
    Error,
    /// Logs only warn and error
    Warn,
    /// Logs everything
    Info,
}

impl TryFrom<&OsStr> for LogLevel {
    type Error = OsString;

    fn try_from(s: &OsStr) -> Result<Self, <Self as TryFrom<&OsStr>>::Error> {
        if s == "error" {
            Ok(LogLevel::Error)
        } else if s == "warn" {
            Ok(LogLevel::Warn)
        } else if s == "info" {
            Ok(LogLevel::Info)
        } else {
            let mut err = OsString::from("Unknown log-level: ");
            err.push(s);
            Err(err)
        }
    }
}

/// Synchronized progress bar and status message printing.
pub struct ProgressOutput {
    quiet: AtomicBool,
    log_level: AtomicU8,
}

impl ProgressOutput {
    /// Returns a new ProgressOutput
    pub const fn new() -> Self {
        Self {
            quiet: AtomicBool::new(false),
            log_level: AtomicU8::new(LogLevel::Info as u8),
        }
    }

    /// Print the given message.
    fn message(&self, message: &str) {
        eprintln!("{}", message);
    }

    /// Returns whether it should silence stdout or not
    pub fn quiet(&self) -> bool {
        self.quiet.load(Ordering::SeqCst)
    }

    /// Causes it to silence stdout
    pub fn set_quiet(&self, quiet: bool) {
        self.quiet.store(quiet, Ordering::SeqCst);
    }

    /// Returns whether the specified log level is enabled or not
    pub fn is_log_enabled(&self, level: LogLevel) -> bool {
        (level as u8) <= self.log_level.load(Ordering::SeqCst)
    }

    /// Sets the log level for wasm-pack
    pub fn set_log_level(&self, log_level: LogLevel) {
        self.log_level.store(log_level as u8, Ordering::SeqCst);
    }

    /// Add an informational message.
    pub fn info(&self, message: &str) {
        if !self.quiet() && self.is_log_enabled(LogLevel::Info) {
            let info = format!("{}: {}", style("[INFO]").bold().dim(), message,);
            self.message(&info);
        }
    }

    /// Add a warning message.
    pub fn warn(&self, message: &str) {
        if !self.quiet() && self.is_log_enabled(LogLevel::Warn) {
            let warn = format!(
                "{}: {} {}",
                style("[WARN]").bold().dim(),
                emoji::WARN,
                message
            );
            self.message(&warn);
        }
    }

    /// Add an error message.
    pub fn error(&self, message: &str) {
        if self.is_log_enabled(LogLevel::Error) {
            let err = format!(
                "{}: {} {}",
                style("[ERR]").bold().dim(),
                emoji::ERROR,
                message
            );
            self.message(&err);
        }
    }
}

impl Default for ProgressOutput {
    fn default() -> Self {
        ProgressOutput::new()
    }
}
