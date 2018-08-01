//! CLI command structures, parsing, and execution.

mod build;
mod login;
mod pack;
mod publish;
pub mod utils;

use self::build::{Build, BuildOptions};
use self::login::login;
use self::pack::pack;
use self::publish::publish;
use error::Error;
use slog::Logger;
use std::path::PathBuf;
use std::result;
use PBAR;

/// The various kinds of commands that `wasm-pack` can execute.
#[derive(Debug, StructOpt)]
pub enum Command {
    /// 🏗️  build your npm package!
    #[structopt(name = "build", alias = "init")]
    Build(BuildOptions),

    #[structopt(name = "pack")]
    /// 🍱  create a tar of your npm package but don't publish!
    Pack {
        /// The path to the Rust crate.
        #[structopt(parse(from_os_str))]
        path: Option<PathBuf>,
    },

    #[structopt(name = "publish")]
    /// 🎆  pack up your npm package and publish!
    Publish {
        /// The path to the Rust crate.
        #[structopt(parse(from_os_str))]
        path: Option<PathBuf>,
    },

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

/// Run a command with the given logger!
pub fn run_wasm_pack(command: Command, log: &Logger) -> result::Result<(), Error> {
    // Run the correct command based off input and store the result of it so that we can clear
    // the progress bar then return it
    let status = match command {
        Command::Build(build_opts) => {
            info!(&log, "Running build command...");
            Build::try_from_opts(build_opts).and_then(|mut b| b.run(&log))
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
