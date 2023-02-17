//! Utilities for managing child processes.
//!
//! This module helps us ensure that all child processes that we spawn get
//! properly logged and their output is logged as well.

use crate::install::Tool;
use anyhow::{bail, Result};
use log::info;
use std::process::{Command, Stdio};

/// Return a new Command object
pub fn new_command(program: &str) -> Command {
    // On Windows, initializes launching <program> as `cmd /c <program>`.
    // Initializing only with `Command::new("npm")` will launch
    //   `npm` with quotes, `"npm"`, causing a run-time error on Windows.
    // See rustc: #42436, #42791, #44542

    if cfg!(windows) {
        let mut cmd = Command::new("cmd");
        cmd.arg("/c").arg(program);
        cmd
    } else {
        Command::new(program)
    }
}

/// Run the given command and return on success.
pub fn run(mut command: Command, command_name: &str) -> Result<()> {
    info!("Running {:?}", command);

    let status = command.status()?;

    if status.success() {
        Ok(())
    } else {
        bail!(
            "failed to execute `{}`: exited with {}\n  full command: {:?}",
            command_name,
            status,
            command,
        )
    }
}

/// Run the given command and return its stdout.
pub fn run_capture_stdout(mut command: Command, command_name: &Tool) -> Result<String> {
    info!("Running {:?}", command);

    let output = command
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        bail!(
            "failed to execute `{}`: exited with {}\n  full command: {:?}",
            command_name,
            output.status,
            command,
        )
    }
}
