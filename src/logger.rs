//! Logging facilities for `wasm-pack`.

use command::Command;
use failure;
use slog::{Drain, Level, Logger};
use slog_async::Async;
use slog_term::{FullFormat, PlainDecorator};
use std::fs::OpenOptions;
use std::path::PathBuf;

/// Create the logger for wasm-pack that will output any info warning or errors we encounter
pub fn new(cmd: &Command, verbosity: u8) -> Result<Logger, failure::Error> {
    let log_path = log_file_path(&cmd);
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    let decorator = PlainDecorator::new(file);
    let drain = FullFormat::new(decorator).build().fuse();

    // Set the log level based off the number of v passed in to the command line args.
    // Level level means only messages of that level and higher are logged. If we have
    // an error then we'll log it unconditionally, but extra levels are only available
    // with extra v
    let log_level = match verbosity {
        0 => Level::Error,
        1 => Level::Info,
        2 => Level::Debug,
        _ => Level::Trace,
    };
    let drain = Async::new(drain).build().filter_level(log_level).fuse();
    Ok(Logger::root(drain, o!()))
}

/// Figure out where to stick the log based off the command arguments given
fn log_file_path(cmd: &Command) -> PathBuf {
    let path = match cmd {
        Command::Build(build_opts) => &build_opts.path,
        Command::Pack { path } => path,
        Command::Publish { path, access: _ } => path,
        Command::Test(test_opts) => &test_opts.path,
        Command::Login { .. } => &None,
    };

    // If the path exists attempt to use it, if not default to the current directory
    if let Some(ref path) = path {
        let mut path_buf = PathBuf::from(path);
        path_buf.push("Cargo.toml");

        // If the manifest file exists put the log in that directory otherwise default
        // to the current directory.
        if path_buf.exists() {
            path_buf.pop();
            path_buf.push("wasm-pack.log");
            path_buf
        } else {
            let mut path_buf = this_dir();
            path_buf.push("wasm-pack.log");
            path_buf
        }
    } else {
        let mut path_buf = this_dir();
        path_buf.push("wasm-pack.log");
        path_buf
    }
}

/// Return a `PathBuf` for the current directory
fn this_dir() -> PathBuf {
    PathBuf::from(".")
}
