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

// This file contains the implementation of the `init` subcommand. This will
// add the wasm32-unknown-unknown target using rustup, compile the crate to
// wasm, create and prepare a `pkg` directory, install wasm-bindgen-cli, and
// then run `wasm-bindgen`.

impl Context {
    /// Run the `init` command for the context.
    pub fn init(&mut self) -> Result<(), Error> {
        let started = Instant::now();

        self.add_wasm32_target()?;
        self.compile_to_wasm()?;
        self.create_pkg_dir()?;
        self.write_package_json()?;
        self.copy_readme_from_crate()?;
        self.install_bindgen()?;
        self.bind()?;

        self.pbar.message(&format!(
            "{} Done in {}",
            emoji::SPARKLE,
            HumanDuration(started.elapsed())
        ));
        self.pbar.message(&format!(
            "{} Your WASM pkg is ready to publish at {}/pkg",
            emoji::PACKAGE,
            &self.path,
        ));
        Ok(())
    }

    /// Add the wasm32-unknown-unknown target using rustup.
    fn add_wasm32_target(&self) -> Result<(), Error> {
        let step = format!(
            "{} {}Adding WASM target...",
            style("[1/7]").bold().dim(),
            emoji::TARGET
        );
        let pb = self.pbar.message(&step);
        rustup_add_wasm_target()?;
        pb.finish();
        Ok(())
    }

    /// Compile the crate using rustc, targeting wasm32-unknown-unknown.
    fn compile_to_wasm(&mut self) -> Result<(), Error> {
        let step = format!(
            "{} {}Compiling to WASM...",
            style("[2/7]").bold().dim(),
            emoji::CYCLONE
        );
        let pb = self.pbar.message(&step);
        let build_res = cargo_build_wasm(&self.path);
        pb.finish();
        build_res
    }

    /// Create a `pkg` directory.
    fn create_pkg_dir(&self) -> Result<(), Error> {
        let step = format!(
            "{} {}Creating a pkg directory...",
            style("[3/7]").bold().dim(),
            emoji::FOLDER
        );
        let pb = self.pbar.message(&step);
        let pkg_dir_path = format!("{}/pkg", self.path);
        create_dir_all(pkg_dir_path)?;
        pb.finish();
        Ok(())
    }

    /// Write the contents of the `package.json`.
    fn write_package_json(&self) -> Result<(), Error> {
        let step = format!(
            "{} {}Writing a package.json...",
            style("[4/7]").bold().dim(),
            emoji::MEMO
        );

        let warn_fmt = |field| {
            format!(
                "Field {} is missing from Cargo.toml. It is not necessary, but recommended",
                field
            )
        };

        let pb = self.pbar.message(&step);
        let pkg_file_path = format!("{}/pkg/package.json", &self.path);
        let mut pkg_file = File::create(pkg_file_path)?;
        let manifest = read_cargo_toml(&self.path)?;
        let npm_data = NpmPackage::new(&manifest, &self.scope);

        if npm_data.description.is_none() {
            self.pbar.warn(&warn_fmt("description"));
        }
        if npm_data.repository.is_none() {
            self.pbar.warn(&warn_fmt("repository"));
        }
        if npm_data.license.is_none() {
            self.pbar.warn(&warn_fmt("license"));
        }

        let npm_json = serde_json::to_string_pretty(&npm_data)?;
        pkg_file.write_all(npm_json.as_bytes())?;
        pb.finish();
        Ok(())
    }

    /// Copy the `README` from the crate into the `pkg` directory.
    fn copy_readme_from_crate(&self) -> Result<(), Error> {
        let step = format!(
            "{} {}Copying over your README...",
            style("[5/7]").bold().dim(),
            emoji::DANCERS
        );
        let pb = self.pbar.message(&step);
        let crate_readme_path = format!("{}/README.md", self.path);
        let new_readme_path = format!("{}/pkg/README.md", self.path);
        if let Err(_) = copy(&crate_readme_path, &new_readme_path) {
            self.pbar.warn("origin crate has no README");
        };
        pb.finish();
        Ok(())
    }

    /// Install `wasm-bindgen-cli`.
    fn install_bindgen(&self) -> Result<(), Error> {
        let step = format!(
            "{} {}Installing WASM-bindgen...",
            style("[6/7]").bold().dim(),
            emoji::DOWN_ARROW
        );
        let pb = self.pbar.message(&step);
        let res = cargo_install_wasm_bindgen();
        pb.finish();
        res
    }

    /// Run `wasm-bindgen-cli`.
    fn bind(&mut self) -> Result<(), Error> {
        let step = format!(
            "{} {}Running WASM-bindgen...",
            style("[7/7]").bold().dim(),
            emoji::RUNNER
        );
        let pb = self.pbar.message(&step);

        // FIXUP: Not entirely a fan of this cloning, but it avoids
        // borrowing problems? There might be a better way around this.
        let crate_path = &self.path.clone();
        let crate_name = &self.manifest().package.name.clone();
        let bind_result = bindgen::wasm_bindgen_build(crate_path, crate_name);

        pb.finish();
        bind_result
    }

}
