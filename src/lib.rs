//! Your favorite rust -> wasm workflow tool!

#![deny(missing_docs)]

extern crate anyhow;
extern crate cargo_metadata;
extern crate console;
extern crate glob;
extern crate parking_lot;
extern crate semver;
extern crate serde;
extern crate strsim;
extern crate which;
#[macro_use]
extern crate serde_derive;
extern crate serde_ignored;
extern crate serde_json;
#[macro_use]
extern crate structopt;
extern crate binary_install;
extern crate chrono;
extern crate curl;
extern crate dialoguer;
extern crate log;
extern crate toml;
extern crate walkdir;

pub mod bindgen;
pub mod build;
pub mod cache;
pub mod child;
pub mod command;
pub mod emoji;
pub mod generate;
pub mod install;
mod installer;
pub mod license;
pub mod lockfile;
pub mod manifest;
pub mod npm;
pub mod progressbar;
pub mod readme;
pub mod stamps;
pub mod target;
pub mod test;
pub mod wasm_opt;

use crate::progressbar::{LogLevel, ProgressOutput};
use crate::{build::WasmPackVersion, command::run_wasm_pack};
use anyhow::Result;
use std::env;
use std::ffi::OsStr;
use std::panic;
use std::sync::mpsc;
use std::thread;
use structopt::StructOpt;

/// The global progress bar and user-facing message output.
pub static PBAR: ProgressOutput = ProgressOutput::new();

/// 📦 ✨  pack and publish your wasm!
#[derive(Debug, StructOpt)]
pub struct Cli {
    /// The subcommand to run.
    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    pub cmd: command::Command,

    /// Log verbosity is based off the number of v used
    #[structopt(long = "verbose", short = "v", parse(from_occurrences))]
    pub verbosity: u8,

    #[structopt(long = "quiet", short = "q")]
    /// No output printed to stdout
    pub quiet: bool,

    #[structopt(long = "log-level", default_value = "info", parse(try_from_os_str = TryFrom::<&OsStr>::try_from))]
    /// The maximum level of messages that should be logged by wasm-pack. [possible values: info, warn, error]
    pub log_level: LogLevel,
}

fn background_check_for_updates() -> mpsc::Receiver<Result<WasmPackVersion>> {
    let (sender, receiver) = mpsc::channel();

    let _detached_thread = thread::spawn(move || {
        let wasm_pack_version = build::check_wasm_pack_versions();

        if let Ok(wasm_pack_version) = wasm_pack_version {
            if !wasm_pack_version.local.is_empty()
                && !wasm_pack_version.latest.is_empty()
                && wasm_pack_version.local.as_bytes() != wasm_pack_version.latest
            {
                let _ = sender.send(Ok(wasm_pack_version));
            }
        } else {
            let _ = sender.send(wasm_pack_version);
        }
    });

    receiver
}

/// Runs the CLI
pub fn main(args: impl Iterator<Item = std::ffi::OsString>) {
    let _ = env_logger::try_init();

    setup_panic_hooks();

    if let Err(e) = run(args) {
        eprintln!("Error: {}", e);
        for cause in e.chain() {
            eprintln!("Caused by: {}", cause);
        }
        ::std::process::exit(1);
    }
}

fn run(cmd_args: impl Iterator<Item = std::ffi::OsString>) -> Result<()> {
    let wasm_pack_version = background_check_for_updates();

    // Deprecate `init`
    if Some(std::ffi::OsStr::new("init")) == env::args_os().nth(1).as_deref() {
        println!("wasm-pack init is deprecated, consider using wasm-pack build");
    }

    if let Ok(me) = env::current_exe() {
        // If we're actually running as the installer then execute our
        // self-installation, otherwise just continue as usual.
        if me
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("executable should have a filename")
            .starts_with("wasm-pack-init")
        {
            installer::install();
        }
    }

    let args = Cli::from_iter(cmd_args);

    PBAR.set_log_level(args.log_level);

    if args.quiet {
        PBAR.set_quiet(true);
    }

    run_wasm_pack(args.cmd)?;

    if let Ok(wasm_pack_version) = wasm_pack_version.try_recv() {
        match wasm_pack_version {
            Ok(wasm_pack_version) =>
                PBAR.warn(&format!("There's a newer version of wasm-pack available, the new version is: {}, you are using: {}. \
                To update, navigate to: https://rustwasm.github.io/wasm-pack/installer/", String::from_utf8_lossy(&wasm_pack_version.latest), wasm_pack_version.local)),
            Err(err) => PBAR.warn(&format!("{}", err))
        }
    }
    Ok(())
}

fn setup_panic_hooks() {
    let meta = human_panic::Metadata {
        version: env!("CARGO_PKG_VERSION").into(),
        name: env!("CARGO_PKG_NAME").into(),
        authors: env!("CARGO_PKG_AUTHORS").replace(':', ", ").into(),
        homepage: env!("CARGO_PKG_HOMEPAGE").into(),
    };

    let default_hook = panic::take_hook();

    if env::var_os("RUST_BACKTRACE").is_none() {
        panic::set_hook(Box::new(move |info: &panic::PanicInfo| {
            // First call the default hook that prints to standard error.
            default_hook(info);

            // Then call human_panic.
            let file_path = human_panic::handle_dump(&meta, info);
            human_panic::print_msg(file_path, &meta)
                .expect("human-panic: printing error message to console failed");
        }));
    }
}
