#[macro_use]
extern crate quicli;
extern crate serde_json;
extern crate toml;

use quicli::prelude::*;

use std::fs::File;
use std::io::prelude::*;

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

#[derive(Deserialize)]
pub struct CargoManifest {
    package: CargoPackage,
}

#[derive(Deserialize)]
pub struct CargoPackage {
    name: String,
    description: String,
    version: String,
}

#[derive(Serialize)]
pub struct NpmPackage {
    name: String,
    description: String,
    version: String,
}

fn read_cargo_toml() -> Result<CargoManifest> {
    let mut cargo_file = File::open("Cargo.toml")?;
    let mut cargo_contents = String::new();
    cargo_file.read_to_string(&mut cargo_contents)?;

    Ok(toml::from_str(&cargo_contents)?)
}

impl CargoManifest {
    fn into_npm(self) -> NpmPackage {
        NpmPackage {
            name: self.package.name,
            description: self.package.description,
            version: self.package.version,
        }
    }
}

fn write_package_json() -> Result<()> {
    let mut pkg_file = File::create("package.json")?;
    let crate_data = read_cargo_toml()?;
    let npm_data = crate_data.into_npm();
    let npm_json = serde_json::to_string(&npm_data)?;
    pkg_file.write_all(npm_json.as_bytes())?;
    Ok(())
}

main!(|args: Cli, log_level: verbosity| match args.cmd {
    Command::Init { .. } => {
        write_package_json()?;
        println!("✍️  wrote a package.json!");
    }
    Command::Pack { .. } => {
        println!("🎒  packed up your pacakge!");
    }
    Command::Publish { .. } => {
        println!("💥  published your package!");
    }
});
