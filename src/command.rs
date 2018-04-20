use bindgen;
use build;
use console::style;
use emoji;
use error::Error;
use indicatif::HumanDuration;
use manifest;
use npm;
#[allow(unused)]
use quicli::prelude::*;
use readme;
use std::fs;
use std::result;
use std::time::Instant;
use PBAR;

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "init")]
    /// 🐣  initialize a package.json based on your compiled wasm
    Init {
        path: Option<String>,
        #[structopt(long = "scope", short = "s")]
        scope: Option<String>,
    },

    #[structopt(name = "pack")]
    /// 🍱  create a tar of your npm package but don't publish! [NOT IMPLEMENTED]
    Pack { path: Option<String> },

    #[structopt(name = "publish")]
    /// 🎆  pack up your npm package and publish! [NOT IMPLEMENTED]
    Publish { path: Option<String> },

    #[structopt(name = "login", alias = "adduser", alias = "add-user")]
    /// 👤  Add a registry user account! (aliases: adduser, add-user) [NOT IMPLEMENTED]
    Login {
        #[structopt(long = "registry", short = "r")]
        /// Default: 'https://registry.npmjs.org/'.
        /// The base URL of the npm package registry. If scope is also specified, this registry will only be used for packages with that scope. scope defaults to the scope of the project directory you're currently in, if any.
        registry: Option<String>,

        #[structopt(long = "scope", short = "s")]
        /// Default: none.
        /// If specified, the user and login credentials given will be associated with the specified scope.
        scope: Option<String>,

        #[structopt(long = "always-auth", short = "a")]
        /// If specified, save configuration indicating that all requests to the given registry should include authorization information. Useful for private registries. Can be used with --registry and / or --scope
        always_auth: bool,

        #[structopt(long = "auth-type", short = "t")]
        /// Default: 'legacy'.
        /// Type: 'legacy', 'sso', 'saml', 'oauth'.
        /// What authentication strategy to use with adduser/login. Some npm registries (for example, npmE) might support alternative auth strategies besides classic username/password entry in legacy npm.
        auth_type: Option<String>,
    },
}

pub fn run_wasm_pack(command: Command) -> result::Result<(), Error> {
    // Run the correct command based off input and store the result of it so that we can clear
    // the progress bar then return it
    let status = match command {
        Command::Init { path, scope } => init(path, scope),
        Command::Pack { path } => pack(path),
        Command::Publish { path } => publish(path),
        Command::Login {
            registry,
            scope,
            always_auth,
            auth_type,
        } => login(registry, scope, always_auth, auth_type),
    };

    match status {
        Ok(_) => {}
        Err(ref e) => {
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
pub fn create_pkg_dir(path: &str) -> result::Result<(), Error> {
    let step = format!(
        "{} {}Creating a pkg directory...",
        style("[3/7]").bold().dim(),
        emoji::FOLDER
    );
    let pb = PBAR.message(&step);
    let pkg_dir_path = format!("{}/pkg", path);
    fs::create_dir_all(pkg_dir_path)?;
    pb.finish();
    Ok(())
}

fn init(path: Option<String>, scope: Option<String>) -> result::Result<(), Error> {
    let started = Instant::now();

    let crate_path = set_crate_path(path);

    build::rustup_add_wasm_target()?;
    build::cargo_build_wasm(&crate_path)?;
    create_pkg_dir(&crate_path)?;
    manifest::write_package_json(&crate_path, scope)?;
    readme::copy_from_crate(&crate_path)?;
    bindgen::cargo_install_wasm_bindgen()?;
    let name = manifest::get_crate_name(&crate_path)?;
    bindgen::wasm_bindgen_build(&crate_path, &name)?;
    PBAR.message(&format!(
        "{} Done in {}",
        emoji::SPARKLE,
        HumanDuration(started.elapsed())
    ));
    PBAR.message(&format!(
        "{} Your WASM pkg is ready to publish at {}/pkg",
        emoji::PACKAGE,
        &crate_path
    ));
    Ok(())
}

fn pack(path: Option<String>) -> result::Result<(), Error> {
    let crate_path = set_crate_path(path);

    npm::npm_pack(&crate_path)?;
    PBAR.message("🎒  packed up your package!");
    Ok(())
}

fn publish(path: Option<String>) -> result::Result<(), Error> {
    let crate_path = set_crate_path(path);

    npm::npm_publish(&crate_path)?;
    PBAR.message("💥  published your package!");
    Ok(())
}

fn login(
    registry: Option<String>,
    scope: Option<String>,
    always_auth: bool,
    auth_type: Option<String>,
) -> result::Result<(), Error> {
    npm::npm_login(registry, scope, always_auth, auth_type)?;

    PBAR.one_off_message("👋  logged you in!");
    Ok(())
}

fn set_crate_path(path: Option<String>) -> String {
    let crate_path = match path {
        Some(p) => p,
        None => ".".to_string(),
    };

    crate_path
}
