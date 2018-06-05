use bindgen;
use build;
use emoji;
use error::Error;
use indicatif::HumanDuration;
use manifest;
use npm;
use progressbar::Step;
#[allow(unused)]
use readme;
use slog::Logger;
use std::fs;
use std::result;
use std::time::Instant;
use PBAR;

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "init")]
    /// 🐣  initialize a package.json based on your compiled wasm!
    Init {
        path: Option<String>,

        #[structopt(long = "scope", short = "s")]
        scope: Option<String>,

        #[structopt(long = "--skip-build")]
        /// Do not build, only update metadata
        skip_build: bool,

        #[structopt(long = "no-typescript")]
        /// By default a *.d.ts file is generated for the generated JS file, but
        /// this flag will disable generating this TypeScript file.
        disable_dts: bool,

        #[structopt(long = "target", short = "t", default_value = "browser")]
        /// Sets the target environment. [possible values: browser, nodejs]
        target: String,

        #[structopt(long = "debug")]
        /// Build without --release.
        debug: bool,
    },

    #[structopt(name = "pack")]
    /// 🍱  create a tar of your npm package but don't publish!
    Pack { path: Option<String> },

    #[structopt(name = "publish")]
    /// 🎆  pack up your npm package and publish!
    Publish { path: Option<String> },

    #[structopt(name = "login", alias = "adduser", alias = "add-user")]
    /// 👤  Add a registry user account! (aliases: adduser, add-user)
    Login {
        #[structopt(long = "registry", short = "r")]
        /// Default: 'https://registry.npmjs.org/'.
        /// The base URL of the npm package registry. If scope is also
        /// specified, this registry will only be used for packages with that
        /// scope. scope defaults to the scope of the project directory you're
        /// currently in, if any.
        registry: Option<String>,

        #[structopt(long = "scope", short = "s")]
        /// Default: none.
        /// If specified, the user and login credentials given will be
        /// associated with the specified scope.
        scope: Option<String>,

        #[structopt(long = "always-auth", short = "a")]
        /// If specified, save configuration indicating that all requests to the
        /// given registry should include authorization information. Useful for
        /// private registries. Can be used with --registry and / or --scope
        always_auth: bool,

        #[structopt(long = "auth-type", short = "t")]
        /// Default: 'legacy'.
        /// Type: 'legacy', 'sso', 'saml', 'oauth'.
        /// What authentication strategy to use with adduser/login. Some npm
        /// registries (for example, npmE) might support alternative auth
        /// strategies besides classic username/password entry in legacy npm.
        auth_type: Option<String>,
    },
}

pub fn run_wasm_pack(command: Command, log: &Logger) -> result::Result<(), Error> {
    // Run the correct command based off input and store the result of it so that we can clear
    // the progress bar then return it
    let status = match command {
        Command::Init {
            path,
            scope,
            skip_build,
            disable_dts,
            target,
            debug,
        } => {
            info!(&log, "Running init command...");
            info!(
                &log,
                "Path: {:?}, Scope: {:?}, Skip build: {}, Disable Dts: {}, Target: {}, Debug: {}",
                &path,
                &scope,
                &skip_build,
                &disable_dts,
                &target,
                debug
            );
            let mode = if skip_build {
                InitMode::Nobuild
            } else {
                InitMode::Normal
            };
            Init::new(path, scope, disable_dts, target, debug).process(&log, mode)
        }
        Command::Pack { path } => {
            info!(&log, "Running pack command...");
            info!(&log, "Path: {:?}", &path);
            pack(path, &log)
        }
        Command::Publish { path } => {
            info!(&log, "Running publish command...");
            info!(&log, "Path: {:?}", &path);
            publish(path, &log)
        }
        Command::Login {
            registry,
            scope,
            always_auth,
            auth_type,
        } => {
            info!(&log, "Running login command...");
            info!(
                &log,
                "Registry: {:?}, Scope: {:?}, Always Auth: {}, Auth Type: {:?}",
                &registry,
                &scope,
                &always_auth,
                &auth_type
            );
            login(registry, scope, always_auth, auth_type, &log)
        }
    };

    match status {
        Ok(_) => {}
        Err(ref e) => {
            error!(&log, "{}", e);
            PBAR.error(e.error_type());
        }
    }

    // Make sure we always clear the progress bar before we abort the program otherwise
    // stderr and stdout output get eaten up and nothing will work. If this part fails
    // to work and clear the progress bars then you're really having a bad day with your tools.
    PBAR.done()?;

    // Return the actual status of the program to the main function
    status
}

// quicli::prelude::* imports a different result struct which gets
// precedence over the std::result::Result, so have had to specify
// the correct type here.
pub fn create_pkg_dir(path: &str, step: &Step) -> result::Result<(), Error> {
    let msg = format!("{}Creating a pkg directory...", emoji::FOLDER);
    let pb = PBAR.step(step, &msg);
    let pkg_dir_path = format!("{}/pkg", path);
    fs::create_dir_all(pkg_dir_path)?;
    pb.finish();
    Ok(())
}

enum InitMode {
    Normal,
    Nobuild,
}

struct Init {
    crate_path: String,
    scope: Option<String>,
    disable_dts: bool,
    target: String,
    debug: bool,
    crate_name: Option<String>,
}

impl Init {
    pub fn new(
        path: Option<String>,
        scope: Option<String>,
        disable_dts: bool,
        target: String,
        debug: bool,
    ) -> Init {
        Init {
            crate_path: set_crate_path(path),
            scope,
            disable_dts,
            target,
            debug,
            crate_name: None,
        }
    }

    pub fn process(&mut self, log: &Logger, mode: InitMode) -> result::Result<(), Error> {
        let process_steps: Vec<fn(&mut Init, &Step, &Logger) -> result::Result<(), Error>> =
            match mode {
                InitMode::Normal => vec![
                    Init::step_check_dependency,
                    Init::step_add_wasm_target,
                    Init::step_build_wasm,
                    Init::step_create_dir,
                    Init::step_create_json,
                    Init::step_copy_readme,
                    Init::step_check_create_type,
                    Init::step_install_wasm_bindgen,
                    Init::step_running_wasm_bindgen,
                ],
                InitMode::Nobuild => vec![
                    Init::step_check_dependency,
                    Init::step_create_dir,
                    Init::step_create_json,
                    Init::step_copy_readme,
                ],
            };
        let mut step_counter = Step::new(process_steps.len());

        let started = Instant::now();

        for process_step in process_steps {
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

    fn step_check_dependency(&mut self, _step: &Step, log: &Logger) -> result::Result<(), Error> {
        info!(&log, "Checking wasm-bindgen dependency...");
        manifest::check_wasm_bindgen(&self.crate_path)?;
        info!(&log, "wasm-bindgen dependency is correctly declared.");
        Ok(())
    }

    fn step_add_wasm_target(&mut self, step: &Step, log: &Logger) -> result::Result<(), Error> {
        info!(&log, "Adding wasm-target...");
        build::rustup_add_wasm_target(step)?;
        info!(&log, "Adding wasm-target was successful.");
        Ok(())
    }

    fn step_build_wasm(&mut self, step: &Step, log: &Logger) -> result::Result<(), Error> {
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

    fn step_create_dir(&mut self, step: &Step, log: &Logger) -> result::Result<(), Error> {
        info!(&log, "Creating a pkg directory...");
        create_pkg_dir(&self.crate_path, step)?;
        info!(&log, "Created a pkg directory at {}.", &self.crate_path);
        Ok(())
    }

    fn step_create_json(&mut self, step: &Step, log: &Logger) -> result::Result<(), Error> {
        info!(&log, "Writing a package.json...");
        manifest::write_package_json(&self.crate_path, &self.scope, self.disable_dts, step)?;
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

    fn step_copy_readme(&mut self, step: &Step, log: &Logger) -> result::Result<(), Error> {
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

    fn step_check_create_type(&mut self, _step: &Step, log: &Logger) -> result::Result<(), Error> {
        info!(&log, "Checking the crate type from the manifest...");
        manifest::check_crate_type(&self.crate_path)?;
        #[cfg(not(target_os = "windows"))]
        info!(
            &log,
            "Checked crate type from the manifest at {}/Cargo.toml.", &self.crate_path
        );
        #[cfg(target_os = "windows")]
        info!(
            &log,
            "Checked crate type from the manifest at {}\\Cargo.toml.", &self.crate_path
        );

        Ok(())
    }

    fn step_install_wasm_bindgen(
        &mut self,
        step: &Step,
        log: &Logger,
    ) -> result::Result<(), Error> {
        info!(&log, "Installing wasm-bindgen-cli...");
        bindgen::cargo_install_wasm_bindgen(step)?;
        info!(&log, "Installing wasm-bindgen-cli was successful.");

        info!(&log, "Getting the crate name from the manifest...");
        self.crate_name = Some(manifest::get_crate_name(&self.crate_path)?);
        #[cfg(not(target_os = "windows"))]
        info!(
            &log,
            "Got crate name {} from the manifest at {}/Cargo.toml.",
            &self.crate_name.as_ref().unwrap(),
            &self.crate_path
        );
        #[cfg(target_os = "windows")]
        info!(
            &log,
            "Got crate name {} from the manifest at {}\\Cargo.toml.",
            &self.crate_name.as_ref().unwrap(),
            &self.crate_path
        );
        Ok(())
    }

    fn step_running_wasm_bindgen(
        &mut self,
        step: &Step,
        log: &Logger,
    ) -> result::Result<(), Error> {
        info!(&log, "Building the wasm bindings...");
        bindgen::wasm_bindgen_build(
            &self.crate_path,
            &self.crate_name.as_ref().unwrap(),
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

fn pack(path: Option<String>, log: &Logger) -> result::Result<(), Error> {
    let crate_path = set_crate_path(path);

    info!(&log, "Packing up the npm package...");
    npm::npm_pack(&crate_path)?;
    #[cfg(not(target_os = "windows"))]
    info!(&log, "Your package is located at {}/pkg", &crate_path);
    #[cfg(target_os = "windows")]
    info!(&log, "Your package is located at {}\\pkg", &crate_path);

    PBAR.message("🎒  packed up your package!");
    Ok(())
}

fn publish(path: Option<String>, log: &Logger) -> result::Result<(), Error> {
    let crate_path = set_crate_path(path);

    info!(&log, "Publishing the npm package...");
    info!(&log, "npm info located in the npm debug log");
    npm::npm_publish(&crate_path)?;
    info!(&log, "Published your package!");

    PBAR.message("💥  published your package!");
    Ok(())
}

fn login(
    registry: Option<String>,
    scope: Option<String>,
    always_auth: bool,
    auth_type: Option<String>,
    log: &Logger,
) -> result::Result<(), Error> {
    let registry = registry.unwrap_or(npm::DEFAULT_NPM_REGISTRY.to_string());

    info!(&log, "Logging in to npm...");
    info!(
        &log,
        "Scope: {:?} Registry: {}, Always Auth: {}, Auth Type: {:?}.",
        &scope,
        &registry,
        always_auth,
        &auth_type
    );
    info!(&log, "npm info located in the npm debug log");
    npm::npm_login(&registry, &scope, always_auth, &auth_type)?;
    info!(&log, "Logged you in!");

    PBAR.message(&format!("👋  logged you in!"));
    Ok(())
}

fn set_crate_path(path: Option<String>) -> String {
    let crate_path = match path {
        Some(p) => p,
        None => ".".to_string(),
    };

    crate_path
}
