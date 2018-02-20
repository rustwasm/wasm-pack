#[macro_use]
extern crate quicli;
extern crate wasm_pack;

use quicli::prelude::*;

/// 📦 ✨  pack and publish your wasm!
#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(subcommand)] // Note that we mark a field as a subcommand
    cmd: Command,
    /// 📝  log all the things!
    #[structopt(long = "verbose", short = "v", parse(from_occurrences))]
    verbosity: u8,
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "init")]
    /// 🐣  initialize a package.json based on your compiled wasm
    Init {},
    #[structopt(name = "pack")]
    /// 🍱  create a tar of your npm package but don't publish!
    Pack {},
    #[structopt(name = "publish")]
    /// 🎆  pack up your npm package and publish!
    Publish {},
}

main!(|args: Cli, log_level: verbosity| match args.cmd {
    Command::Init { .. } => {
        wasm_pack::write_package_json()?;
        println!("✍️  wrote a package.json!");
    }
    Command::Pack { .. } => {
        println!("🎒  packed up your pacakge!");
    }
    Command::Publish { .. } => {
        println!("💥  published your package!");
    }
});
