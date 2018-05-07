use std::fs::File;
use std::io::prelude::*;

use failure::Error;
use serde_json;

use wasm_pack::Cli;
use wasm_pack::command::Command;
use wasm_pack::context::{Action, Context};
use wasm_pack::context::progressbar::ProgressOutput;
use wasm_pack::manifest::{CargoManifest, NpmPackage};

pub fn read_package_json(path: &str) -> Result<NpmPackage, Error> {
    let manifest_path = format!("{}/pkg/package.json", path);
    let mut pkg_file = File::open(manifest_path)?;
    let mut pkg_contents = String::new();
    pkg_file.read_to_string(&mut pkg_contents)?;

    Ok(serde_json::from_str(&pkg_contents)?)
}

// FIXUP: These do not need to receive options.

pub fn get_crate_manifest(path: Option<String>) -> CargoManifest {
    let mut context = create_init_context(path); // FIXUP scope?
    *context.manifest().clone()
}

pub fn get_crate_name(path: Option<String>) -> String {
    let mut context = create_init_context(path); // FIXUP scope?
    let manifest = context.manifest();
    manifest.package.name.clone()
}

fn crate_path(path_arg: Option<String>) -> String {
    path_arg.unwrap_or(".".to_string())
}

// FIXUP: Not a giant fan of this.
fn create_init_context(path: Option<String>) -> Context {
    let cli = Cli {
        cmd: Command::Init {
            path,
            scope: None,
        },
        verbosity: 0,
    };
    Context::from(cli)
}
