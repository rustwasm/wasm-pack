//! Implementation of the `wasm-pack build` command.

use bindgen;
use build;
use command::utils::{create_pkg_dir, set_crate_path};
use emoji;
use failure::Error;
use indicatif::HumanDuration;
use license;
use lockfile::Lockfile;
use manifest;
use progressbar::Step;
use readme;
use slog::Logger;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use wasm_pack_binary_install::{Cache, Download};
use PBAR;

/// Everything required to configure and run the `wasm-pack init` command.
#[allow(missing_docs)]
pub struct Build {
    pub crate_path: PathBuf,
    pub crate_data: manifest::CrateData,
    pub scope: Option<String>,
    pub disable_dts: bool,
    pub target: String,
    pub profile: BuildProfile,
    pub mode: BuildMode,
    pub out_dir: PathBuf,
    pub bindgen: Option<Download>,
    pub cache: Cache,
    pub extra_options: Vec<String>,
}

/// The `BuildMode` determines which mode of initialization we are running, and
/// what build and install steps we perform.
#[derive(Clone, Copy, Debug)]
pub enum BuildMode {
    /// Perform all the build and install steps.
    Normal,
    /// Don't install tools like `wasm-bindgen`, just use the global
    /// environment's existing versions to do builds.
    Noinstall,
    /// Skip the rustc version check
    Force,
}

impl Default for BuildMode {
    fn default() -> BuildMode {
        BuildMode::Normal
    }
}

impl FromStr for BuildMode {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "no-install" => Ok(BuildMode::Noinstall),
            "normal" => Ok(BuildMode::Normal),
            "force" => Ok(BuildMode::Force),
            _ => bail!("Unknown build mode: {}", s),
        }
    }
}

/// The build profile controls whether optimizations, debug info, and assertions
/// are enabled or disabled.
#[derive(Clone, Copy, Debug)]
pub enum BuildProfile {
    /// Enable assertions and debug info. Disable optimizations.
    Dev,
    /// Enable optimizations. Disable assertions and debug info.
    Release,
    /// Enable optimizations and debug info. Disable assertions.
    Profiling,
}

/// Everything required to configure and run the `wasm-pack build` command.
#[derive(Debug, StructOpt)]
pub struct BuildOptions {
    /// The path to the Rust crate.
    #[structopt(parse(from_os_str))]
    pub path: Option<PathBuf>,

    /// The npm scope to use in package.json, if any.
    #[structopt(long = "scope", short = "s")]
    pub scope: Option<String>,

    #[structopt(long = "mode", short = "m", default_value = "normal")]
    /// Sets steps to be run. [possible values: no-install, normal, force]
    pub mode: BuildMode,

    #[structopt(long = "no-typescript")]
    /// By default a *.d.ts file is generated for the generated JS file, but
    /// this flag will disable generating this TypeScript file.
    pub disable_dts: bool,

    #[structopt(long = "target", short = "t", default_value = "browser")]
    /// Sets the target environment. [possible values: browser, nodejs, no-modules]
    pub target: String,

    #[structopt(long = "debug")]
    /// Deprecated. Renamed to `--dev`.
    pub debug: bool,

    #[structopt(long = "dev")]
    /// Create a development build. Enable debug info, and disable
    /// optimizations.
    pub dev: bool,

    #[structopt(long = "release")]
    /// Create a release build. Enable optimizations and disable debug info.
    pub release: bool,

    #[structopt(long = "profiling")]
    /// Create a profiling build. Enable optimizations and debug info.
    pub profiling: bool,

    #[structopt(long = "out-dir", short = "d", default_value = "pkg")]
    /// Sets the output directory with a relative path.
    pub out_dir: String,

    #[structopt(last = true)]
    /// List of extra options to pass to `cargo build`
    pub extra_options: Vec<String>,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            path: None,
            scope: None,
            mode: BuildMode::Normal,
            disable_dts: false,
            target: String::new(),
            debug: false,
            dev: false,
            release: false,
            profiling: false,
            out_dir: String::new(),
            extra_options: Vec::new(),
        }
    }
}

type BuildStep = fn(&mut Build, &Step, &Logger) -> Result<(), Error>;

impl Build {
    /// Construct a build command from the given options.
    pub fn try_from_opts(build_opts: BuildOptions) -> Result<Self, Error> {
        let crate_path = set_crate_path(build_opts.path)?;
        let crate_data = manifest::CrateData::new(&crate_path)?;
        let out_dir = crate_path.join(PathBuf::from(build_opts.out_dir));

        let dev = build_opts.dev || build_opts.debug;
        let profile = match (dev, build_opts.release, build_opts.profiling) {
            (false, false, false) | (false, true, false) => BuildProfile::Release,
            (true, false, false) => BuildProfile::Dev,
            (false, false, true) => BuildProfile::Profiling,
            // Unfortunately, `structopt` doesn't expose clap's `conflicts_with`
            // functionality yet, so we have to implement it ourselves.
            _ => bail!("Can only supply one of the --dev, --release, or --profiling flags"),
        };

        // `possible_values` in clap isn't supported by `structopt`
        let possible_targets = ["browser", "nodejs", "no-modules"];
        if !possible_targets.contains(&build_opts.target.as_str()) {
            bail!("Supported targets: browser, nodejs, no-modules");
        }

        Ok(Build {
            crate_path,
            crate_data,
            scope: build_opts.scope,
            disable_dts: build_opts.disable_dts,
            target: build_opts.target,
            profile,
            mode: build_opts.mode,
            out_dir,
            bindgen: None,
            cache: Cache::new()?,
            extra_options: build_opts.extra_options,
        })
    }

    /// Configures the global binary cache used for this build
    pub fn set_cache(&mut self, cache: Cache) {
        self.cache = cache;
    }

    /// Execute this `Build` command.
    pub fn run(&mut self, log: &Logger) -> Result<(), Error> {
        let process_steps = Build::get_process_steps(&self.mode);

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
            "Your wasm pkg is ready to publish at {}.",
            self.out_dir.display()
        );

        PBAR.message(&format!("{} Done in {}", emoji::SPARKLE, &duration));

        PBAR.message(&format!(
            "{} Your wasm pkg is ready to publish at {}.",
            emoji::PACKAGE,
            self.out_dir.display()
        ));
        Ok(())
    }

    fn get_process_steps(mode: &BuildMode) -> Vec<(&'static str, BuildStep)> {
        macro_rules! steps {
            ($($name:ident),+) => {
                {
                let mut steps: Vec<(&'static str, BuildStep)> = Vec::new();
                    $(steps.push((stringify!($name), Build::$name));)*
                        steps
                    }
                };
            ($($name:ident,)*) => (steps![$($name),*])
        }
        match &mode {
            BuildMode::Normal => steps![
                step_check_rustc_version,
                step_check_crate_config,
                step_add_wasm_target,
                step_build_wasm,
                step_create_dir,
                step_create_json,
                step_copy_readme,
                step_copy_license,
                step_install_wasm_bindgen,
                step_run_wasm_bindgen,
            ],
            BuildMode::Noinstall => steps![
                step_check_rustc_version,
                step_check_crate_config,
                step_build_wasm,
                step_create_dir,
                step_create_json,
                step_copy_readme,
                step_copy_license,
                step_run_wasm_bindgen
            ],
            BuildMode::Force => steps![
                step_build_wasm,
                step_create_dir,
                step_create_json,
                step_copy_readme,
                step_copy_license,
                step_run_wasm_bindgen
            ],
        }
    }

    fn step_check_rustc_version(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Checking rustc version...");
        let version = build::check_rustc_version(step)?;
        let msg = format!("rustc version is {}.", version);
        info!(&log, "{}", &msg);
        Ok(())
    }

    fn step_check_crate_config(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Checking crate configuration...");
        self.crate_data.check_crate_config(step)?;
        info!(&log, "Crate is correctly configured.");
        Ok(())
    }

    fn step_add_wasm_target(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Adding wasm-target...");
        build::rustup_add_wasm_target(log, step)?;
        info!(&log, "Adding wasm-target was successful.");
        Ok(())
    }

    fn step_build_wasm(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Building wasm...");
        build::cargo_build_wasm(
            log,
            &self.crate_path,
            self.profile,
            step,
            &self.extra_options,
        )?;

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
        create_pkg_dir(&self.out_dir, step)?;
        info!(&log, "Created a pkg directory at {:#?}.", &self.crate_path);
        Ok(())
    }

    fn step_create_json(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Writing a package.json...");
        self.crate_data.write_package_json(
            &self.out_dir,
            &self.scope,
            self.disable_dts,
            &self.target,
            step,
        )?;
        info!(
            &log,
            "Wrote a package.json at {:#?}.",
            &self.out_dir.join("package.json")
        );
        Ok(())
    }

    fn step_copy_readme(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Copying readme from crate...");
        readme::copy_from_crate(&self.crate_path, &self.out_dir, step)?;
        info!(&log, "Copied readme from crate to {:#?}.", &self.out_dir);
        Ok(())
    }

    fn step_copy_license(&mut self, step: &Step, log: &Logger) -> Result<(), failure::Error> {
        info!(&log, "Copying license from crate...");
        license::copy_from_crate(&self.crate_data, &self.crate_path, &self.out_dir, step)?;
        info!(&log, "Copied license from crate to {:#?}.", &self.out_dir);
        Ok(())
    }

    fn step_install_wasm_bindgen(
        &mut self,
        step: &Step,
        log: &Logger,
    ) -> Result<(), failure::Error> {
        info!(&log, "Identifying wasm-bindgen dependency...");
        let lockfile = Lockfile::new(&self.crate_data)?;
        let bindgen_version = lockfile.require_wasm_bindgen()?;
        info!(&log, "Installing wasm-bindgen-cli...");
        let install_permitted = match self.mode {
            BuildMode::Normal => true,
            BuildMode::Force => true,
            BuildMode::Noinstall => false,
        };
        let bindgen = bindgen::install_wasm_bindgen(
            &self.cache,
            &bindgen_version,
            install_permitted,
            step,
            log,
        )?;
        self.bindgen = Some(bindgen);
        info!(&log, "Installing wasm-bindgen-cli was successful.");
        Ok(())
    }

    fn step_run_wasm_bindgen(&mut self, step: &Step, log: &Logger) -> Result<(), Error> {
        info!(&log, "Building the wasm bindings...");
        bindgen::wasm_bindgen_build(
            &self.crate_data,
            self.bindgen.as_ref().unwrap(),
            &self.out_dir,
            self.disable_dts,
            &self.target,
            self.profile,
            step,
            log,
        )?;
        info!(&log, "wasm bindings were built at {:#?}.", &self.out_dir);
        Ok(())
    }
}
