pub mod init;
mod login;
mod pack;
mod publish;
pub mod utils;

use self::init::{Init, InitMode};
use self::login::login;
use self::pack::pack;
use self::publish::publish;
use error::Error;
use slog::Logger;
use std::result;
use PBAR;

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "init")]
    /// 🐣  initialize a package.json based on your compiled wasm!
    Init {
        path: Option<String>,

        #[structopt(long = "scope", short = "s")]
        scope: Option<String>,

        #[structopt(long = "mode", short = "m", default_value = "normal")]
        /// Sets steps to be run. [possible values: no-build, no-install, normal]
        mode: String,

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
            mode,
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
                &mode,
                &disable_dts,
                &target,
                debug
            );
            let modetype = match &*mode {
                "no-build" => InitMode::Nobuild,
                "no-install" => InitMode::Noinstall,
                "normal" => InitMode::Normal,
                _ => InitMode::Normal,
            };
            Init::new(path, scope, disable_dts, target, debug)?.process(&log, modetype)
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
            PBAR.error(e.error_type())?;
        }
    }

    // Return the actual status of the program to the main function
    status
}
