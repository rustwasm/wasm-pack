use bindgen;
use build;
use console::style;
use emoji;
use error::Error;
use indicatif::HumanDuration;
use manifest;
use npm;
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

        #[structopt(long = "no-typescript")]
        /// By default a *.d.ts file is generated for the generated JS file, but
        /// this flag will disable generating this TypeScript file.
        disable_dts: bool,

        #[structopt(long = "target", short = "t", default_value = "browser")]
        /// Sets the target environment. [possible values: browser, nodejs]
        target: String,
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
            disable_dts,
            target,
        } => {
            info!(&log, "Running init command...");
            info!(
                &log,
                "Path: {:?}, Scope: {:?}, Disable Dts: {}, Target: {}",
                &path,
                &scope,
                &disable_dts,
                &target
            );
            init(path, scope, disable_dts, target, &log)
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

    // Return the actual status of the program to the main function
    status
}

// quicli::prelude::* imports a different result struct which gets
// precedence over the std::result::Result, so have had to specify
// the correct type here.
pub fn create_pkg_dir(path: &str) -> result::Result<(), Error> {
    let step = format!(
        "{} {}Creating a pkg directory...",
        style("[3/7]").bold().dim(),
        emoji::FOLDER
    );
    PBAR.message(&step);
    let pkg_dir_path = format!("{}/pkg", path);
    fs::create_dir_all(pkg_dir_path)?;
    Ok(())
}

fn init(
    path: Option<String>,
    scope: Option<String>,
    disable_dts: bool,
    target: String,
    log: &Logger,
) -> result::Result<(), Error> {
    let started = Instant::now();

    let crate_path = set_crate_path(path);

    info!(&log, "Adding wasm-target...");
    build::rustup_add_wasm_target()?;
    info!(&log, "Adding wasm-target was successful.");

    info!(&log, "Building wasm...");
    build::cargo_build_wasm(&crate_path)?;

    #[cfg(not(target_os = "windows"))]
    info!(
        &log,
        "wasm built at {}/target/wasm32-unknown-unknown/release.", &crate_path
    );
    #[cfg(target_os = "windows")]
    info!(
        &log,
        "wasm built at {}\\target\\wasm32-unknown-unknown\\release.", &crate_path
    );

    info!(&log, "Creating a pkg directory...");
    create_pkg_dir(&crate_path)?;
    info!(&log, "Created a pkg directory at {}.", &crate_path);

    info!(&log, "Writing a package.json...");
    manifest::write_package_json(&crate_path, scope, disable_dts)?;
    #[cfg(not(target_os = "windows"))]
    info!(
        &log,
        "Wrote a package.json at {}/pkg/package.json.", &crate_path
    );
    #[cfg(target_os = "windows")]
    info!(
        &log,
        "Wrote a package.json at {}\\pkg\\package.json.", &crate_path
    );

    info!(&log, "Copying readme from crate...");
    readme::copy_from_crate(&crate_path)?;
    #[cfg(not(target_os = "windows"))]
    info!(&log, "Copied readme from crate to {}/pkg.", &crate_path);
    #[cfg(target_os = "windows")]
    info!(&log, "Copied readme from crate to {}\\pkg.", &crate_path);

    info!(&log, "Installing wasm-bindgen-cli...");
    bindgen::cargo_install_wasm_bindgen()?;
    info!(&log, "Installing wasm-bindgen-cli was successful.");

    info!(&log, "Getting the crate name from the manifest...");
    let name = manifest::get_crate_name(&crate_path)?;
    #[cfg(not(target_os = "windows"))]
    info!(
        &log,
        "Got crate name {} from the manifest at {}/Cargo.toml.", &name, &crate_path
    );
    #[cfg(target_os = "windows")]
    info!(
        &log,
        "Got crate name {} from the manifest at {}\\Cargo.toml.", &name, &crate_path
    );

    info!(&log, "Building the wasm bindings...");
    bindgen::wasm_bindgen_build(&crate_path, &name, disable_dts, target)?;
    #[cfg(not(target_os = "windows"))]
    info!(&log, "wasm bindings were built at {}/pkg.", &crate_path);
    #[cfg(target_os = "windows")]
    info!(&log, "wasm bindings were built at {}\\pkg.", &crate_path);

    let duration = HumanDuration(started.elapsed());
    info!(&log, "Done in {}.", &duration);
    info!(
        &log,
        "Your WASM pkg is ready to publish at {}/pkg.", &crate_path
    );

    PBAR.message(&format!("{} Done in {}", emoji::SPARKLE, &duration));

    PBAR.message(&format!(
        "{} Your WASM pkg is ready to publish at {}/pkg.",
        emoji::PACKAGE,
        &crate_path
    ));
    Ok(())
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
