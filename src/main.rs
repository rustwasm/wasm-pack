extern crate failure;
#[macro_use]
extern crate human_panic;
extern crate indicatif;
extern crate structopt;
extern crate wasm_pack;

use failure::Fail;
use structopt::StructOpt;
use wasm_pack::command::run_wasm_pack;
use wasm_pack::error::Error;
use wasm_pack::Cli;

fn main() {
    setup_panic!();
    if let Err(e) = run() {
        eprintln!("{}", e);
        for cause in e.causes().skip(1) {
            eprintln!("Caused by: {}", cause);
        }
        ::std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let args = Cli::from_args();
    run_wasm_pack(args.cmd)?;
    Ok(())
}
