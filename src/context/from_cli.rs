use std::convert::From;

use super::{Action, Context};
use command::Command;
use context::progressbar::ProgressOutput;
use Cli;

impl From<Cli> for Context {
    fn from(args: Cli) -> Context {
        let pbar = ProgressOutput::new();
        let Cli { cmd, verbosity } = args;
        match cmd {
            Command::Init { path, scope } => Context {
                action: Action::Init,
                manifest: None,
                path: crate_path(path),
                pbar,
                scope: scope.clone(),
                _verbosity: verbosity,
            },
            Command::Pack { path } => Context {
                action: Action::Pack,
                manifest: None,
                path: crate_path(path),
                pbar,
                scope: None,
                _verbosity: verbosity,
            },
            Command::Publish { path } => Context {
                action: Action::Publish,
                manifest: None,
                path: crate_path(path),
                pbar,
                scope: None,
                _verbosity: verbosity,
            },
        }
    }
}

fn crate_path(path_arg: Option<String>) -> String {
    path_arg.unwrap_or(".".to_string())
}
