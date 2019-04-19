//! Utilties for managing child processes.
//!
//! This module helps us ensure that all child processes that we spawn get
//! properly logged and their output is logged as well.

use log::info;
use std::io::Error as StdError;
use std::process::{Command, ExitStatus, Stdio};

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

/// Error from running Command processes
/// This captures the standard error output
#[derive(Fail, Debug)]
#[fail(display = "failed to execute `{}`: {}", command_name, fail_reason)]
pub struct CommandError {
    command_name: String,
    fail_reason: String,
    /// the output printed to stderr, if any
    pub stderr: Option<String>,
}

impl CommandError {
    fn from_status(command_name: &str, status: ExitStatus, stderr: Option<String>) -> CommandError {
        CommandError {
            command_name: command_name.to_string(),
            fail_reason: format!("exited with {}", status),
            stderr: stderr,
        }
    }

    fn from_error(command_name: &str, err: StdError) -> CommandError {
        CommandError {
            command_name: command_name.to_string(),
            fail_reason: err.to_string(),
            stderr: None,
        }
    }
}

/// Run the given command and return on success.
pub fn run(mut command: Command, command_name: &str) -> Result<(), CommandError> {
    info!("Running {:?}", command);

    let cmd_output = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output();
    match cmd_output {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                Err(CommandError::from_status(
                    command_name,
                    output.status,
                    Some(String::from_utf8_lossy(&output.stderr).into_owned()),
                ))
            }
        }
        Err(e) => Err(CommandError::from_error(command_name, e)),
    }
}

/// Run the given command and return its stdout.
pub fn run_capture_stdout(
    mut command: Command,
    command_name: &str,
) -> Result<String, CommandError> {
    info!("Running {:?}", command);

    let cmd_output = command.stdin(Stdio::inherit()).output();
    match cmd_output {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).into_owned())
            } else {
                Err(CommandError::from_status(
                    command_name,
                    output.status,
                    Some(String::from_utf8_lossy(&output.stderr).into_owned()),
                ))
            }
        }
        Err(e) => Err(CommandError::from_error(command_name, e)),
    }
}
