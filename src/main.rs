extern crate console;
extern crate failure;
extern crate indicatif;
#[macro_use]
extern crate quicli;
extern crate wasm_pack;

use wasm_pack::progressbar;

use std::time::Instant;

use console::{style, Emoji};
use indicatif::HumanDuration;
use quicli::prelude::*;
use wasm_pack::{bindgen, build, manifest, readme};

static TARGET: Emoji = Emoji("🎯  ", "");
static CYCLONE: Emoji = Emoji("🌀  ", "");
static FOLDER: Emoji = Emoji("📂  ", "");
static MEMO: Emoji = Emoji("📝  ", "");
static DOWN_ARROW: Emoji = Emoji("⬇️  ", "");
static RUNNER: Emoji = Emoji("🏃‍♀️  ", "");
static SPARKLE: Emoji = Emoji("✨ ", ":-)");
static PACKAGE: Emoji = Emoji("📦  ", ":-)");

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
    /// 🍱  create a tar of your npm package but don't publish! [NOT IMPLEMENTED]
    Pack {},
    #[structopt(name = "publish")]
    /// 🎆  pack up your npm package and publish! [NOT IMPLEMENTED]
    Publish {},
}

main!(|args: Cli, log_level: verbosity| match args.cmd {
    Command::Init { path } => {
        let started = Instant::now();

        let crate_path = match path {
            Some(p) => p,
            None => ".".to_string(),
        };

        let step1 = format!(
            "{} {}Adding WASM target...",
            style("[1/7]").bold().dim(),
            TARGET
        );
        let pb1 = progressbar::new(step1);
        build::rustup_add_wasm_target();
        pb1.finish();
        let step2 = format!(
            "{} {}Compiling to WASM...",
            style("[2/7]").bold().dim(),
            CYCLONE
        );
        let pb2 = progressbar::new(step2);
        build::cargo_build_wasm(&crate_path);
        pb2.finish();
        let step3 = format!(
            "{} {}Creating a pkg directory...",
            style("[3/7]").bold().dim(),
            FOLDER
        );
        let pb3 = progressbar::new(step3);
        wasm_pack::create_pkg_dir(&crate_path)?;
        pb3.finish();
        let step4 = format!(
            "{} {}Writing a package.json...",
            style("[4/7]").bold().dim(),
            MEMO
        );
        let pb4 = progressbar::new(step4);
        manifest::write_package_json(&crate_path)?;
        pb4.finish();
        readme::copy_from_crate(&crate_path)?;
        let step6 = format!(
            "{} {}Installing WASM-bindgen...",
            style("[6/7]").bold().dim(),
            DOWN_ARROW
        );
        let pb6 = progressbar::new(step6);
        bindgen::cargo_install_wasm_bindgen();
        pb6.finish();
        let name = manifest::get_crate_name(&crate_path)?;
        let step7 = format!(
            "{} {}Running WASM-bindgen...",
            style("[7/7]").bold().dim(),
            RUNNER
        );
        let pb7 = progressbar::new(step7);
        bindgen::wasm_bindgen_build(&crate_path, &name);
        pb7.finish();
        println!("{} Done in {}", SPARKLE, HumanDuration(started.elapsed()));
        println!(
            "{} Your WASM pkg is ready to publish at {}/pkg",
            PACKAGE, &crate_path
        )
    }
    Command::Pack { .. } => {
        println!("🙅‍♀️  whoops! this is not implemented yet! sorry!");
        //println!("🎒  packed up your package!");
    }
    Command::Publish { .. } => {
        println!("🙅‍♀️  whoops! this is not implemented yet! sorry!");
        //println!("💥  published your package!");
    }
});
