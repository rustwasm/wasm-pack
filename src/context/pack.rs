use std::fs::{copy, create_dir_all, File};
use std::io::prelude::*;
use std::time::Instant;

use bindgen;
use command::{
    cargo_build_wasm, cargo_install_wasm_bindgen, pack, publish, rustup_add_wasm_target,
};
use console::style;
use emoji;
use error::Error;
use indicatif::HumanDuration;
use manifest::{read_cargo_toml, CargoManifest, NpmPackage};
use serde_json;
use toml;

use super::Context;

impl Context {
    pub fn pack(&mut self) -> Result<(), Error> {
        pack(&self.path)
    }
}
