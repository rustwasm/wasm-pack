use std::fs::{copy, create_dir_all, File};
use std::io::prelude::*;
use std::time::Instant;

use bindgen;
use command::{pack, publish, cargo_build_wasm, cargo_install_wasm_bindgen, rustup_add_wasm_target};
use console::style;
use emoji;
use error::Error;
use indicatif::HumanDuration;
use manifest::{read_cargo_toml, CargoManifest, NpmPackage};
use serde_json;
use toml;

mod from_cli;
mod progressbar;

use self::progressbar::ProgressOutput;

pub enum Action {
    // FIXUP: Not sure how to feel about this enum?
    Init,
    Pack,
    Publish,
}

// FIXUP: Cannot derive 'Debug' trait here because 'ProgressOutput' does not derive.
pub struct Context {
    action: Action,
    manifest: Option<CargoManifest>,
    path: String,
    pbar: ProgressOutput,
    scope: Option<String>,
    verbosity: u8,
}

impl Context {
    /// Run the command in the current context.
    pub fn run(&mut self) -> Result<(), Error> {
        let status = match self.action {
            Action::Init => self.init(),
            Action::Pack => self.pack(),
            Action::Publish => self.publish(),
        };

        // FIXUP: A `self.pbar.finish()` might be needed here?

        match status {
            Ok(_) => {}
            Err(ref e) => {
                self.pbar.error(e.error_type());
            }
        }

        // Make sure we always clear the progress bar before we abort the program otherwise
        // stderr and stdout output get eaten up and nothing will work. If this part fails
        // to work and clear the progress bars then you're really having a bad day with your tools.
        self.pbar.done()?;

        // Return the actual status of the program to the main function
        status
    }

    // Lazy `Cargo.toml` manifest reading.
    // ------------------------------------------------------------------------

    /// Return a borrow of the crate manifest. If the manifest has not been
    /// read yet, then read the contents and place them in self.manifest.
    pub fn manifest(&mut self) -> &CargoManifest {
        if self.manifest.is_none() {
            if self.read_manifest(".").is_err() {
                unimplemented!(); // Something bad happened if we are here.
            }
        }

        self.manifest.as_ref().unwrap() // FIXUP: This seems wonky?
    }

    /// Read the contents of `Cargo.toml`, place them into self.manifest.
    fn read_manifest(&mut self, path: &str) -> Result<(), Error> {
        let manifest_path = format!("{}/Cargo.toml", path);
        let mut cargo_file = File::open(manifest_path)?;
        let mut cargo_contents = String::new();
        cargo_file.read_to_string(&mut cargo_contents)?;
        self.manifest = toml::from_str(&cargo_contents)?;
        Ok(())
    }

    // Command Wrappers:
    // ------------------------------------------------------------------------
    // These commands are responsible for wrapping the command functions,
    // printing informational messages to the progress bar, and returning a
    // Result object representing whether or not the operation was successful.
    // ------------------------------------------------------------------------

    // TODO:
    // 1. Add the wasm target. (install::rustup_add_wasm_target)
    // 2. Compile the crate, targeting wasm (build::cargo_build_wasm)
    // 3. Create the npm package directory. (command::create_pkg_dir)
    // 4. Write the package json. (manifest::write_package_json)
    // 5. Copy the crate readme into the package directory. (readme::copy_from_crate)
    // 6. Install wasm-bindgen-cli (install::cargo_install_wasm_bindgen) // FIXUP: Move this up in the process?
    // 7. Run wasm-bindgen, generate bindings. (bindgen::wasm_bindgen_build)
    fn init(&mut self) -> Result<(), Error> {
        let started = Instant::now();

        self.install()?; // 1.
        self.build()?; // 2.
        self.create_pkg_dir()?; // 3.
        self.write_package_json()?; // 4.
        self.copy_readme_from_crate()?; // 5.
        self.install_bindgen()?; // 6.
        self.bind()?; // 7.

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

    // Initialization helper functions: (FIXUP: This could be moved into a different file maybe?)

    /// Install requires dependencies including wasm-bindgen, and add the
    /// wasm32-unknown-unknown target to rustc using rustup.
    fn install(&self) -> Result<(), Error> {
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

    fn build(&mut self) -> Result<(), Error> {
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

    fn bind(&mut self) -> Result<(), Error> {
        let step = format!(
            "{} {}Running WASM-bindgen...",
            style("[7/7]").bold().dim(),
            emoji::RUNNER
        );
        let pb = self.pbar.message(&step);

        // FIXUP: Not entirely a fan of this cloning, but it avoids mutability/borrowing problems? There might be a better way around this.
        let crate_path = &self.path.clone();
        let crate_name = &self.manifest().package.name.clone();
        let bind_result = bindgen::wasm_bindgen_build(crate_path, crate_name);

        pb.finish();
        bind_result
    }

    // ------------------------------------------------------------------------

    fn pack(&mut self) -> Result<(), Error> {
        pack(&self.path)
    }

    fn publish(&mut self) -> Result<(), Error> {
        publish(&self.path)
    }
}
