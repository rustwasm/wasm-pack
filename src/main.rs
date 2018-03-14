extern crate wasm_pack;

extern crate indicatif;
#[macro_use]
extern crate quicli;

use std::time::Instant;

use indicatif::HumanDuration;
use quicli::prelude::*;
use wasm_pack::{bindgen, build, emoji, manifest, readme};

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
    Init {
        path: Option<String>,
        #[structopt(long = "scope", short = "s")]
        scope: Option<String>
    },
    #[structopt(name = "pack")]
    /// 🍱  create a tar of your npm package but don't publish! [NOT IMPLEMENTED]
    Pack {},
    #[structopt(name = "publish")]
    /// 🎆  pack up your npm package and publish! [NOT IMPLEMENTED]
    Publish {},
}

main!(|args: Cli, log_level: verbosity| match args.cmd {
    Command::Init { path, scope } => {
        let started = Instant::now();

        let crate_path = match path {
            Some(p) => p,
            None => ".".to_string(),
        };

        build::rustup_add_wasm_target();
        build::cargo_build_wasm(&crate_path);
        wasm_pack::create_pkg_dir(&crate_path)?;
        manifest::write_package_json(&crate_path, scope)?;
        readme::copy_from_crate(&crate_path)?;
        bindgen::cargo_install_wasm_bindgen();
        let name = manifest::get_crate_name(&crate_path)?;
        bindgen::wasm_bindgen_build(&crate_path, &name);
        println!(
            "{} Done in {}",
            emoji::SPARKLE,
            HumanDuration(started.elapsed())
        );
        println!(
            "{} Your WASM pkg is ready to publish at {}/pkg",
            emoji::PACKAGE,
            &crate_path
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
