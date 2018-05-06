use std::convert::From;

use super::{Action, Context};
use command::Command;
use progressbar::ProgressOutput;
use Cli;

// FIXUP: Would using TryFrom here make sense?

impl From<Cli> for Context {
    fn from(args: Cli) -> Context {
        let pbar = ProgressOutput::new();
        let Cli {
            cmd,
            verbosity,
        } = args;
        match cmd {
            Command::Init { path, scope } => Context {
                action: Action::Init,
                manifest: None,
                path: path.clone().unwrap_or(".".to_string()),
                pbar,
                scope: scope.clone(),
                verbosity,
            },
            Command::Pack { path } => Context {
                action: Action::Pack,
                manifest: None,
                path: path.clone().unwrap_or(".".to_string()),
                pbar,
                scope: None,
                verbosity,
            },
            Command::Publish { path } => Context {
                action: Action::Publish,
                manifest: None,
                path: path.clone().unwrap_or(".".to_string()),
                pbar,
                scope: None,
                verbosity,
            }
        }
    }
}
