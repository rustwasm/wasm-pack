#[macro_use]
extern crate quicli;
extern crate wasm_pack;

mod build;
mod bindgen;

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
    Init { path: Option<String> },
    #[structopt(name = "pack")]
    /// 🍱  create a tar of your npm package but don't publish!
    Pack {},
    #[structopt(name = "publish")]
    /// 🎆  pack up your npm package and publish!
    Publish {},
}

main!(|args: Cli, log_level: verbosity| match args.cmd {
    Command::Init { path } => {
        let crate_path = match path {
            Some(p) => p,
            None => ".".to_string(),
        };
        build::rustup_add_wasm_target();
        build::cargo_build_wasm(&crate_path);
        wasm_pack::write_package_json(&crate_path)?;
        bindgen::cargo_install_wasm_bindgen();
        let name = wasm_pack::get_crate_name(&crate_path)?;
        bindgen::wasm_bindgen_build(&crate_path, &name);
    }
    Command::Pack { .. } => {
        println!("🎒  packed up your package!");
    }
    Command::Publish { .. } => {
        println!("💥  published your package!");
    }
});
