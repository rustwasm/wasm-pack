//! Initializing a crate for packing `.wasm`s.

use command::utils::{create_pkg_dir, set_crate_path};
use emoji;
use error::Error;
use indicatif::HumanDuration;
use manifest;
use progressbar::Step;
use readme;
use slog::Logger;
use std::path::PathBuf;
use std::time::Instant;
use PBAR;

/// Everything required to configure and run the `wasm-pack init` command.
pub struct Init {
    crate_path: PathBuf,
    scope: Option<String>,
    disable_dts: bool,
    target: String,
}

/// `Init` options
#[derive(Debug, StructOpt)]
pub struct InitOptions {
    /// The path to the Rust crate.
    #[structopt(parse(from_os_str))]
    pub path: Option<PathBuf>,

    /// The npm scope to use in package.json, if any.
    #[structopt(long = "scope", short = "s")]
    pub scope: Option<String>,

    #[structopt(long = "no-typescript")]
    /// By default a *.d.ts file is generated for the generated JS file, but
    /// this flag will disable generating this TypeScript file.
    pub disable_dts: bool,

    #[structopt(long = "target", short = "t", default_value = "browser")]
    /// Sets the target environment. [possible values: browser, nodejs]
    pub target: String,
}

impl From<InitOptions> for Init {
    fn from(init_opts: InitOptions) -> Self {
        let crate_path = set_crate_path(init_opts.path);
        Init {
            crate_path,
            scope: init_opts.scope,
            disable_dts: init_opts.disable_dts,
            target: init_opts.target,
        }
    }
}

type InitStep = fn(&mut Init, &Step, &Logger) -> Result<(), Error>;

impl Init {
    /// Execute this `Init` command.
    pub fn run(&mut self, log: &Logger) -> Result<(), Error> {
        let process_steps = Init::set_process_steps();

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

    fn set_process_steps() -> Vec<(&'static str, InitStep)> {
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
        steps![step_create_dir, step_create_json, step_copy_readme,]
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
}
