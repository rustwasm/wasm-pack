//! Utilties for managing child processes.
//!
//! This module helps us ensure that:
//!
//! * All child processes that we spawn get properly logged and their output is
//!   logged as well.
//!
//! * That any "quick running" child process does not spam the console with its
//!   output.
//!
//! * That any "long running" child process gets its output copied to our
//!   stderr, so that the user isn't sitting there wondering if anything at all
//!   is happening. This is important for showing `cargo build`'s output, for
//!   example.

use error::Error;
use failure;
use slog::Logger;
use std::{
    io::{self, BufRead},
    process,
    sync::mpsc,
    thread, time,
};
use PBAR;

fn taking_too_long(since: time::Instant) -> bool {
    since.elapsed() > time::Duration::from_millis(200)
}

enum Output<S: AsRef<str>> {
    Stdout(S),
    Stderr(S),
}

fn print_child_output<S>(out: Result<Output<S>, io::Error>, command_name: &str)
where
    S: AsRef<str>,
{
    let message = match out {
        Ok(Output::Stdout(line)) => format!("{} (stdout): {}", command_name, line.as_ref()),
        Ok(Output::Stderr(line)) => format!("{} (stderr): {}", command_name, line.as_ref()),
        Err(e) => format!("error reading {} output: {}", command_name, e),
    };
    PBAR.message(&message);
}

fn handle_output<I, S>(
    output: I,
    logger: &Logger,
    should_print: bool,
    stdout: &mut String,
    stderr: &mut String,
    command_name: &str,
) where
    I: IntoIterator<Item = Result<Output<S>, io::Error>>,
    S: AsRef<str>,
{
    for out in output {
        match out {
            Ok(Output::Stdout(ref line)) => {
                let line = line.as_ref().trim_end();
                info!(logger, "{} (stdout): {}", command_name, line);
                stdout.push_str(line);
            }
            Ok(Output::Stderr(ref line)) => {
                let line = line.as_ref().trim_end();
                info!(logger, "{} (stderr): {}", command_name, line);
                stderr.push_str(line);
            }
            Err(ref e) => {
                warn!(logger, "error reading output: {}", e);
            }
        }
        if should_print {
            print_child_output(out, command_name);
        }
    }
}

/// Run the given command and return its stdout.
///
/// If the command takes "too long", then its stdout and stderr are also piped
/// to our stdout and stderr so that the user has an idea of what is going on
/// behind the scenes.
pub fn run(
    logger: &Logger,
    mut command: process::Command,
    command_name: &str,
) -> Result<String, failure::Error> {
    info!(logger, "Running {:?}", command);

    let mut child = command
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn()?;

    let since = time::Instant::now();

    let stdout = io::BufReader::new(child.stdout.take().unwrap());
    let stderr = io::BufReader::new(child.stderr.take().unwrap());

    let (send, recv) = mpsc::channel();
    let stdout_send = send.clone();
    let stderr_send = send;

    // Because pipes have a fixed-size buffer, we need to keep reading stdout
    // and stderr on a separate thread to avoid potential dead locks with
    // waiting on the child process.

    let stdout_handle = thread::spawn(move || {
        for line in stdout.lines() {
            stdout_send.send(line.map(Output::Stdout)).unwrap();
        }
    });

    let stderr_handle = thread::spawn(move || {
        for line in stderr.lines() {
            stderr_send.send(line.map(Output::Stderr)).unwrap();
        }
    });

    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut is_long_running = false;

    loop {
        if !is_long_running && taking_too_long(since) {
            // The command has now been taking too long. Print the buffered stdout and
            // stderr, and then continue waiting on the child.
            stdout
                .lines()
                .map(|l| Ok(Output::Stdout(l)))
                .chain(stderr.lines().map(|l| Ok(Output::Stderr(l))))
                .for_each(|l| print_child_output(l, command_name));

            is_long_running = true;
        }

        // Get any output that's been sent on the channel without blocking.
        handle_output(
            recv.try_iter(),
            logger,
            is_long_running,
            &mut stdout,
            &mut stderr,
            command_name,
        );

        if let Some(exit) = child.try_wait()? {
            // Block on collecting the rest of the child's output.
            handle_output(
                recv,
                logger,
                is_long_running,
                &mut stdout,
                &mut stderr,
                command_name,
            );

            // Join the threads reading the child's output to make sure the
            // finish OK.
            stdout_handle.join().unwrap();
            stderr_handle.join().unwrap();

            if exit.success() {
                return Ok(stdout);
            } else {
                let msg = format!("`{}` did not exit successfully", command_name);
                return Err(Error::cli(&msg, stderr.into(), exit).into());
            }
        }

        thread::yield_now();
    }
}
