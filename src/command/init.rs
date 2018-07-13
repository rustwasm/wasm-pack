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
use std::time::Instant;
use PBAR;

pub fn create_pkg_dir(path: &str, step: &Step) -> Result<(), Error> {
    let msg = format!("{}Creating a pkg directory...", emoji::FOLDER);
    PBAR.step(step, &msg);
    let pkg_dir_path = format!("{}/pkg", path);
    fs::create_dir_all(pkg_dir_path)?;
    Ok(())
}

pub enum InitMode {
    Normal,
    Nobuild,
    Noinstall,
}

pub struct Init {
    crate_path: String,
    scope: Option<String>,
    disable_dts: bool,
    target: String,
    debug: bool,
    crate_name: String,
}

type InitStep = fn(&mut Init, &Step, &Logger) -> Result<(), Error>;

impl Init {
    pub fn new(
        path: Option<String>,
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
            "Your WASM pkg is ready to publish at {}/pkg.", &self.crate_path
        );

        PBAR.message(&format!("{} Done in {}", emoji::SPARKLE, &duration));

        PBAR.message(&format!(
            "{} Your WASM pkg is ready to publish at {}/pkg.",
            emoji::PACKAGE,
            &self.crate_path
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

        #[cfg(not(target_os = "windows"))]
        info!(
            &log,
            "wasm built at {}/target/wasm32-unknown-unknown/release.", &self.crate_path
        );
        #[cfg(target_os = "windows")]
        info!(
            &log,
            "wasm built at {}\\target\\wasm32-unknown-unknown\\release.", &self.crate_path
        );
        Ok(())
    }

    fn step_create_dir(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Creating a pkg directory...");
        create_pkg_dir(&self.crate_path, step)?;
        info!(&log, "Created a pkg directory at {}.", &self.crate_path);
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
        #[cfg(not(target_os = "windows"))]
        info!(
            &log,
            "Wrote a package.json at {}/pkg/package.json.", &self.crate_path
        );
        #[cfg(target_os = "windows")]
        info!(
            &log,
            "Wrote a package.json at {}\\pkg\\package.json.", &self.crate_path
        );
        Ok(())
    }

    fn step_copy_readme(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Copying readme from crate...");
        readme::copy_from_crate(&self.crate_path, step)?;
        #[cfg(not(target_os = "windows"))]
        info!(
            &log,
            "Copied readme from crate to {}/pkg.", &self.crate_path
        );
        #[cfg(target_os = "windows")]
        info!(
            &log,
            "Copied readme from crate to {}\\pkg.", &self.crate_path
        );
        Ok(())
    }

    fn step_install_wasm_bindgen(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Installing wasm-bindgen-cli...");
        bindgen::cargo_install_wasm_bindgen(step)?;
        info!(&log, "Installing wasm-bindgen-cli was successful.");

        info!(&log, "Getting the crate name from the manifest...");
        self.crate_name = manifest::get_crate_name(&self.crate_path)?;
        #[cfg(not(target_os = "windows"))]
        info!(
            &log,
            "Got crate name {} from the manifest at {}/Cargo.toml.",
            &self.crate_name,
            &self.crate_path
        );
        #[cfg(target_os = "windows")]
        info!(
            &log,
            "Got crate name {} from the manifest at {}\\Cargo.toml.",
            &self.crate_name,
            &self.crate_path
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
        #[cfg(not(target_os = "windows"))]
        info!(
            &log,
            "wasm bindings were built at {}/pkg.", &self.crate_path
        );
        #[cfg(target_os = "windows")]
        info!(
            &log,
            "wasm bindings were built at {}\\pkg.", &self.crate_path
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
