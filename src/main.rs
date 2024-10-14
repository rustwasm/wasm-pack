#![allow(clippy::redundant_closure, clippy::redundant_pattern_matching)]

extern crate anyhow;
extern crate clap;
extern crate env_logger;
extern crate human_panic;
extern crate log;
extern crate wasm_pack;
extern crate which;

use anyhow::Result;
use clap::Parser;
use std::env;
use std::panic;
use std::sync::mpsc;
use std::thread;
use wasm_pack::{
    build::{self, WasmPackVersion},
    command::run_wasm_pack,
    Cli, PBAR,
};

mod installer;

fn background_check_for_updates() -> mpsc::Receiver<Result<WasmPackVersion>> {
    let (sender, receiver) = mpsc::channel();

    let _detached_thread = thread::spawn(move || {
        let wasm_pack_version = build::check_wasm_pack_versions();

        if let Ok(wasm_pack_version) = wasm_pack_version {
            if !wasm_pack_version.local.is_empty()
                && !wasm_pack_version.latest.is_empty()
                && wasm_pack_version.local != wasm_pack_version.latest
            {
                let _ = sender.send(Ok(wasm_pack_version));
            }
        } else {
            let _ = sender.send(wasm_pack_version);
        }
    });

    receiver
}

fn main() {
    env_logger::init();

    setup_panic_hooks();

    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        for cause in e.chain() {
            eprintln!("Caused by: {}", cause);
        }
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let wasm_pack_version = background_check_for_updates();

    // Deprecate `init`
    if let Some("init") = env::args().nth(1).as_ref().map(|arg| arg.as_str()) {
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

    let args = Cli::parse();

    PBAR.set_log_level(args.log_level);

    if args.quiet {
        PBAR.set_quiet(true);
    }

    run_wasm_pack(args.cmd)?;

    if let Ok(wasm_pack_version) = wasm_pack_version.try_recv() {
        match wasm_pack_version {
            Ok(wasm_pack_version) =>
                PBAR.warn(&format!("There's a newer version of wasm-pack available, the new version is: {}, you are using: {}. \
                To update, navigate to: https://rustwasm.github.io/wasm-pack/installer/", wasm_pack_version.latest, wasm_pack_version.local)),
            Err(err) => PBAR.warn(&format!("{}", err))
        }
    }

    Ok(())
}

fn setup_panic_hooks() {
    let meta = human_panic::Metadata {
        version: env!("CARGO_PKG_VERSION").into(),
        name: env!("CARGO_PKG_NAME").into(),
        authors: env!("CARGO_PKG_AUTHORS").replace(":", ", ").into(),
        homepage: env!("CARGO_PKG_HOMEPAGE").into(),
    };

    let default_hook = panic::take_hook();

    if let Err(_) = env::var("RUST_BACKTRACE") {
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
