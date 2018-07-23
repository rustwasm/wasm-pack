//! Initializing a crate for packing `.wasm`s.

use bindgen;
use build;
use command::utils::set_crate_path;
use emoji;
use error::Error;
use indicatif::HumanDuration;
use manifest;
use progressbar::Step;
use readme;
use slog::Logger;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use PBAR;

/// Construct our `pkg` directory in the crate.
pub fn create_pkg_dir(path: &Path, step: &Step) -> Result<(), Error> {
    let msg = format!("{}Creating a pkg directory...", emoji::FOLDER);
    PBAR.step(step, &msg);
    let pkg_dir_path = path.join("pkg");
    fs::create_dir_all(pkg_dir_path)?;
    Ok(())
}

/// The `InitMode` determines which mode of initialization we are running, and
/// what build and install steps we perform.
pub enum InitMode {
    /// Perform all the build and install steps.
    Normal,
    /// Don't build the crate as a `.wasm` but do install tools and create
    /// meta-data.
    Nobuild,
    /// Don't install tools like `wasm-bindgen`, just use the global
    /// environment's existing versions to do builds.
    Noinstall,
}

/// Everything required to configure and run the `wasm-pack init` command.
pub struct Init {
    crate_path: PathBuf,
    scope: Option<String>,
    disable_dts: bool,
    target: String,
    debug: bool,
    crate_name: String,
}

type InitStep = fn(&mut Init, &Step, &Logger) -> Result<(), Error>;

impl Init {
    /// Construct a new `Init` command.
    pub fn new(
        path: Option<PathBuf>,
        scope: Option<String>,
        disable_dts: bool,
        target: String,
        debug: bool,
    ) -> Result<Init, Error> {
        let crate_path = set_crate_path(path);
        let crate_name = manifest::get_crate_name(&crate_path)?;
        Ok(Init {
            crate_path,
            scope,
            disable_dts,
            target,
            debug,
            crate_name,
        })
    }

    fn get_process_steps(mode: InitMode) -> Vec<(&'static str, InitStep)> {
        macro_rules! steps {
            ($($name:ident),+) => {
                {
                    let mut steps: Vec<(&'static str, InitStep)> = Vec::new();
                    $(steps.push((stringify!($name), Init::$name));)*
                    steps
                }
            };
            ($($name:ident,)*) => (steps![$($name),*])
        }

        match mode {
            InitMode::Normal => steps![
                step_check_crate_config,
                step_add_wasm_target,
                step_build_wasm,
                step_create_dir,
                step_create_json,
                step_copy_readme,
                step_install_wasm_bindgen,
                step_run_wasm_bindgen,
            ],
            InitMode::Nobuild => steps![step_create_dir, step_create_json, step_copy_readme,],
            InitMode::Noinstall => steps![
                step_check_crate_config,
                step_build_wasm,
                step_create_dir,
                step_create_json,
                step_copy_readme,
                step_run_wasm_bindgen
            ],
        }
    }

    /// Execute this `Init` command.
    pub fn process(&mut self, log: &Logger, mode: InitMode) -> Result<(), Error> {
        let process_steps = Init::get_process_steps(mode);

        let mut step_counter = Step::new(process_steps.len());

        let started = Instant::now();

        for (_, process_step) in process_steps {
            process_step(self, &step_counter, log)?;
            step_counter.inc();
        }

        let duration = HumanDuration(started.elapsed());
        info!(&log, "Done in {}.", &duration);
        info!(
            &log,
            "Your WASM pkg is ready to publish at {:#?}.",
            &self.crate_path.join("pkg")
        );

        PBAR.message(&format!("{} Done in {}", emoji::SPARKLE, &duration));

        PBAR.message(&format!(
            "{} Your WASM pkg is ready to publish at {:#?}.",
            emoji::PACKAGE,
            &self.crate_path.join("pkg")
        ));
        Ok(())
    }

    fn step_check_crate_config(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Checking crate configuration...");
        manifest::check_crate_config(&self.crate_path, step)?;
        info!(&log, "Crate is correctly configured.");
        Ok(())
    }

    fn step_add_wasm_target(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Adding wasm-target...");
        build::rustup_add_wasm_target(step)?;
        info!(&log, "Adding wasm-target was successful.");
        Ok(())
    }

    fn step_build_wasm(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Building wasm...");
        build::cargo_build_wasm(&self.crate_path, self.debug, step)?;

        info!(
            &log,
            "wasm built at {:#?}.",
            &self
                .crate_path
                .join("target")
                .join("wasm32-unknown-unknown")
                .join("release")
        );
        Ok(())
    }

    fn step_create_dir(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Creating a pkg directory...");
        create_pkg_dir(&self.crate_path, step)?;
        info!(&log, "Created a pkg directory at {:#?}.", &self.crate_path);
        Ok(())
    }

    fn step_create_json(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Writing a package.json...");
        manifest::write_package_json(
            &self.crate_path,
            &self.scope,
            self.disable_dts,
            &self.target,
            step,
        )?;
        info!(
            &log,
            "Wrote a package.json at {:#?}.",
            &self.crate_path.join("pkg").join("package.json")
        );
        Ok(())
    }

    fn step_copy_readme(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Copying readme from crate...");
        readme::copy_from_crate(&self.crate_path, step)?;
        info!(
            &log,
            "Copied readme from crate to {:#?}.",
            &self.crate_path.join("pkg")
        );
        Ok(())
    }

    fn step_install_wasm_bindgen(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Installing wasm-bindgen-cli...");
        bindgen::cargo_install_wasm_bindgen(step)?;
        info!(&log, "Installing wasm-bindgen-cli was successful.");

        info!(&log, "Getting the crate name from the manifest...");
        self.crate_name = manifest::get_crate_name(&self.crate_path)?;
        info!(
            &log,
            "Got crate name {:#?} from the manifest at {:#?}.",
            &self.crate_name,
            &self.crate_path.join("Cargo.toml")
        );
        Ok(())
    }

    fn step_run_wasm_bindgen(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Building the wasm bindings...");
        bindgen::wasm_bindgen_build(
            &self.crate_path,
            &self.crate_name,
            self.disable_dts,
            &self.target,
            self.debug,
            step,
        )?;
        info!(
            &log,
            "wasm bindings were built at {:#?}.",
            &self.crate_path.join("pkg")
        );
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn init_normal_build() {
        let steps: Vec<&str> = Init::get_process_steps(InitMode::Normal)
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        assert_eq!(
            steps,
            [
                "step_check_crate_config",
                "step_add_wasm_target",
                "step_build_wasm",
                "step_create_dir",
                "step_create_json",
                "step_copy_readme",
                "step_install_wasm_bindgen",
                "step_run_wasm_bindgen"
            ]
        );
    }

    #[test]
    fn init_skip_build() {
        let steps: Vec<&str> = Init::get_process_steps(InitMode::Nobuild)
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        assert_eq!(
            steps,
            ["step_create_dir", "step_create_json", "step_copy_readme"]
        );
    }

    #[test]
    fn init_skip_install() {
        let steps: Vec<&str> = Init::get_process_steps(InitMode::Noinstall)
            .into_iter()
            .map(|(n, _)| n)
            .collect();
        assert_eq!(
            steps,
            [
                "step_check_crate_config",
                "step_build_wasm",
                "step_create_dir",
                "step_create_json",
                "step_copy_readme",
                "step_run_wasm_bindgen"
            ]
        );
    }
}
