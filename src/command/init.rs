use indicatif::HumanDuration;
use std::result;
use std::time::Instant;

use error::Error;

use bindgen;
use emoji;
use manifest;
use readme;
use PBAR;

use super::build::{cargo_build_wasm, rustup_add_wasm_target};
use super::create_pkg_dir;

pub fn init(crate_path: &str, scope: &Option<String>) -> result::Result<(), Error> {
    rustup_add_wasm_target()?;
    cargo_build_wasm(&crate_path)?;
    create_pkg_dir(&crate_path)?;
    manifest::write_package_json(&crate_path, scope)?;
    readme::copy_from_crate(&crate_path)?;
    bindgen::cargo_install_wasm_bindgen()?;
    let name = manifest::get_crate_name(&crate_path)?;
    bindgen::wasm_bindgen_build(&crate_path, &name)?;
    Ok(())
}
