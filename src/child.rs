//! Utilties for managing child processes.
//!
//! This module helps us ensure that all child processes that we spawn get
//! properly logged and their output is logged as well.

use error::Error;
use failure;
use slog::Logger;
use std::{
    io::{self, Read},
    mem, process, string,
    sync::mpsc,
    thread,
};
use PBAR;

#[derive(Debug)]
enum OutputFragment {
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
}

/// Read data from the give reader and send it as an `OutputFragment` over the
/// given sender.
fn read_and_send<R, F>(
    mut reader: R,
    sender: mpsc::Sender<OutputFragment>,
    mut map: F,
) -> io::Result<()>
where
    R: Read,
    F: FnMut(Vec<u8>) -> OutputFragment,
{
    let mut buf = vec![0; 1024];
    loop {
        match reader.read(&mut buf) {
            Err(e) => {
                if e.kind() == io::ErrorKind::Interrupted {
                    continue;
                } else {
                    return Err(e);
                }
            }
            Ok(0) => return Ok(()),
            Ok(n) => {
                buf.truncate(n);
                let buf = mem::replace(&mut buf, vec![0; 1024]);
                sender.send(map(buf)).unwrap();
            }
        }
    }
}

/// Accumulates output from a stream of output fragments and calls a callback on
/// each complete line as it is accumulating.
struct OutputAccumulator<F> {
    result: String,
    in_progress: Vec<u8>,
    on_each_line: F,
}

impl<F> OutputAccumulator<F>
where
    F: FnMut(&str),
{
    /// Construct a new output accumulator with the given `on_each_line`
    /// callback.
    fn new(on_each_line: F) -> OutputAccumulator<F> {
        OutputAccumulator {
            result: String::new(),
            in_progress: Vec::new(),
            on_each_line,
        }
    }

    /// Add another fragment of output to the accumulation, calling the
    /// `on_each_line` callback for any complete lines we accumulate.
    fn push(&mut self, fragment: Vec<u8>) -> Result<(), string::FromUtf8Error> {
        debug_assert!(!fragment.is_empty());
        self.in_progress.extend(fragment);

        if let Some((last_newline, _)) = self
            .in_progress
            .iter()
            .cloned()
            .enumerate()
            .rev()
            .find(|(_, ch)| *ch == b'\n')
        {
            let next_in_progress: Vec<u8> = self.in_progress[last_newline + 1..]
                .iter()
                .cloned()
                .collect();
            let mut these_lines = mem::replace(&mut self.in_progress, next_in_progress);
            these_lines.truncate(last_newline + 1);
            let these_lines = String::from_utf8(these_lines)?;
            for line in these_lines.lines() {
                (self.on_each_line)(line);
            }
            self.result.push_str(&these_lines);
        }

        Ok(())
    }

    /// Finish accumulation, run the `on_each_line` callback on the final line
    /// (if any), and return the accumulated output.
    fn finish(mut self) -> Result<String, string::FromUtf8Error> {
        if !self.in_progress.is_empty() {
            let last_line = String::from_utf8(self.in_progress)?;
            (self.on_each_line)(&last_line);
            self.result.push_str(&last_line);
        }
        Ok(self.result)
    }
}

/// Run the given command and return its stdout.
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

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let (send, recv) = mpsc::channel();
    let stdout_send = send.clone();
    let stderr_send = send;

    // Because pipes have a fixed-size buffer, we need to keep reading stdout
    // and stderr on a separate thread to avoid potential dead locks with
    // waiting on the child process.

    let stdout_handle =
        thread::spawn(move || read_and_send(stdout, stdout_send, OutputFragment::Stdout));
    let stderr_handle =
        thread::spawn(move || read_and_send(stderr, stderr_send, OutputFragment::Stderr));

    let mut stdout = OutputAccumulator::new(|line| {
        info!(logger, "{} (stdout): {}", command_name, line);
        PBAR.message(line)
    });
    let mut stderr = OutputAccumulator::new(|line| {
        info!(logger, "{} (stderr): {}", command_name, line);
        PBAR.message(line)
    });

    for output in recv {
        match output {
            OutputFragment::Stdout(line) => stdout.push(line)?,
            OutputFragment::Stderr(line) => stderr.push(line)?,
        };
    }

    let stdout = stdout.finish()?;
    let stderr = stderr.finish()?;

    // Join the threads reading the child's output to make sure the finish OK.
    stdout_handle.join().unwrap()?;
    stderr_handle.join().unwrap()?;

    let exit = child.wait()?;
    if exit.success() {
        return Ok(stdout);
    } else {
        let msg = format!("`{}` did not exit successfully", command_name);
        return Err(Error::cli(&msg, stdout.into(), stderr.into(), exit).into());
    }
}
