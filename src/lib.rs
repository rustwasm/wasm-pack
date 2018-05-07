extern crate console;
#[macro_use]
extern crate failure;
extern crate indicatif;
extern crate quicli;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

// #[macro_use]
// extern crate lazy_static; // FIXUP: lazy_static is not needed now?

pub mod command;
pub mod context;
pub mod emoji;
pub mod error;
pub mod init;
pub mod manifest;
pub mod pack;
pub mod publish;

use quicli::prelude::*;

/// 📦 ✨  pack and publish your wasm!
#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    pub cmd: command::Command,
    ///  log all the things
    #[structopt(long = "verbose", short = "v", parse(from_occurrences))]
    pub verbosity: u8,
}
