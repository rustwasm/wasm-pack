use std::fs::File;
use std::io::prelude::*;

use failure::Error;
use serde_json;

use wasm_pack::context::{Action, Context};
use wasm_pack::context::progressbar::ProgressOutput;
use wasm_pack::manifest::NpmPackage;

pub fn read_package_json(path: &str) -> Result<NpmPackage, Error> {
    let manifest_path = format!("{}/pkg/package.json", path);
    let mut pkg_file = File::open(manifest_path)?;
    let mut pkg_contents = String::new();
    pkg_file.read_to_string(&mut pkg_contents)?;

    Ok(serde_json::from_str(&pkg_contents)?)
}

pub fn get_crate_name(path: Option<String>) -> String {
    let mut noop_context = Context {
        action: Action::NoOp,
        manifest: None,
        path: crate_path(path),
        pbar: ProgressOutput::new(),
        scope: None,
        _verbosity: 0,
    };
    let manifest = noop_context.manifest();
    manifest.package.name.clone()
}

fn crate_path(path_arg: Option<String>) -> String {
    path_arg.unwrap_or(".".to_string())
}
