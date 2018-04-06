extern crate wasm_pack;

extern crate indicatif;
#[macro_use]
extern crate quicli;

use quicli::prelude::*;
use wasm_pack::Cli;
use wasm_pack::command::{Command, init_command, pack_command, publish_command};


main!(|args: Cli, log_level: verbosity| match args.cmd {
    Command::Init { path, scope } => {
        init_command(path, scope)?;
    }
    Command::Pack { path } => {
        pack_command(path)?;
    }
    Command::Publish { path } => {
        publish_command(path)?;
    }
});
