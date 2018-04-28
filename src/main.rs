extern crate wasm_pack;

extern crate indicatif;
#[macro_use]
extern crate quicli;
#[macro_use]
extern crate human_panic;

use quicli::prelude::*;
use wasm_pack::command::run_wasm_pack;
use wasm_pack::Cli;

main!(|args: Cli, log_level: verbosity| {
    setup_panic!();
    run_wasm_pack(args.cmd)?;
});
