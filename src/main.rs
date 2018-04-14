extern crate wasm_pack;

extern crate indicatif;
#[macro_use]
extern crate quicli;

use quicli::prelude::*;
use wasm_pack::command::{init, pack, publish, Command};
use wasm_pack::Cli;

main!(|args: Cli, log_level: verbosity| match args.cmd {
    Command::Init { path, scope } => {
        init(path, scope)?;
    }
    Command::Pack { path } => {
        pack(path)?;
    }
    Command::Publish { path } => {
        publish(path)?;
    }
});
