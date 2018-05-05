use std::fs::File;
use std::io::prelude::*;

use command::{init, pack, publish, Command};
use error::Error;
use manifest::CargoManifest;
use progressbar::ProgressOutput;
use toml;

mod from_cmd;

pub enum Action {
    Init,
    Pack,
    Publish,
}

// FIXUP: Cannot derive 'Debug' trait because 'ProgressOutput' does not derive.

pub struct Context {
    action: Action,
    manifest: Option<CargoManifest>,
    path: String,
    pbar: ProgressOutput,
    scope: Option<String>,
    // verbosity: u8,   // FIXUP: Once logging is added, this might make sense?
}

impl Context {
    pub fn run(&mut self) -> Result<(), Error> {
        let status = match self.action {
            Action::Init => init(&self.path, &self.scope),
            Action::Pack => pack(&self.path),
            Action::Publish => publish(&self.path),
        };

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
}
