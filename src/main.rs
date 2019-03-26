extern crate atty;
extern crate env_logger;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate human_panic;
extern crate log;
extern crate structopt;
extern crate wasm_pack;
extern crate which;

use std::env;
use std::sync::mpsc;
use std::thread;
use structopt::StructOpt;
use wasm_pack::command::build::Build;
use wasm_pack::{command::run_wasm_pack, Cli};

mod installer;

fn background_check_for_updates() -> mpsc::Receiver<(String, String)> {
    let (sender, receiver) = mpsc::channel();
    let _detached_thread = thread::spawn(move || {
        if let Ok((local, latest)) = Build::return_wasm_pack_versions() {
            if !local.is_empty() && !latest.is_empty() && local != latest {
                sender.send((local, latest)).unwrap();
            }
        }
    });

    receiver
}

fn main() {
    env_logger::init();
    setup_panic!();
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        for cause in e.iter_causes() {
            eprintln!("Caused by: {}", cause);
        }
        ::std::process::exit(1);
    }
}

fn run() -> Result<(), failure::Error> {
    let update_available = background_check_for_updates();

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

    let args = Cli::from_args();
    run_wasm_pack(args.cmd)?;

    if let Ok(update_available) = update_available.try_recv() {
        println!("There's a newer version of wasm-pack available, the new version is: {}, you are using: {}. \
            To update, navigate to: https://rustwasm.github.io/wasm-pack/installer/", update_available.1, update_available.0);
    }

    Ok(())
}
