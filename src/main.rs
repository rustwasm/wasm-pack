extern crate failure;
#[macro_use]
extern crate human_panic;
extern crate structopt;
extern crate wasm_pack;

use failure::Fail;
use std::env;
use structopt::StructOpt;
use wasm_pack::{command::run_wasm_pack, error::Error, logger, Cli};

fn main() {
    setup_panic!();
    if let Err(e) = run() {
        eprintln!("{}", e);
        for cause in Fail::iter_causes(&e) {
            eprintln!("Caused by: {}", cause);
        }
        ::std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    // Deprecate `init`
    if let Some("init") = env::args().nth(1).as_ref().map(|arg| arg.as_str()) {
        println!("wasm-pack init is deprecated, consider using wasm-pack build");
    }

    let args = Cli::from_args();
    let log = logger::new(&args.cmd, args.verbosity)?;
    run_wasm_pack(args.cmd, &log)?;
    Ok(())
}
