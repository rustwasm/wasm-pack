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
    /// ☔  init and pack, but don't publish
    #[structopt(long = "dry-run", short = "d")]
    dry_run: bool,

    /// 📝  log all the things!
    #[structopt(long = "verbose", short = "v", parse(from_occurrences))]
    verbosity: u8,
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

main!(|args: Cli, log_level: verbosity| {
    write_package_json()?;
    println!("✍️  wrote package.json");
    if !args.dry_run {
        println!("⬆️  published pkg to npm");
    }
});
