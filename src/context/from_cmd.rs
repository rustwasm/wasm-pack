use std::convert::From;

use super::{Action, Context};
use command::Command;
use progressbar::ProgressOutput;

// FIXUP: Would using TryFrom here make sense?

impl From<Command> for Context {
    fn from(cmd: Command) -> Context {
        let action = get_cmd_action(&cmd);
        let path = get_cmd_path(&cmd);
        let scope = get_cmd_scope(&cmd);
        Context {
            action,
            manifest: None,
            pbar: ProgressOutput::new(),
            path,
            scope,
        }
    }
}

fn get_cmd_action(cmd: &Command) -> Action {
    match cmd {
        Command::Init { .. } => Action::Init,
        Command::Pack { .. } => Action::Pack,
        Command::Publish { .. } => Action::Publish,
    }
}

fn get_cmd_path(cmd: &Command) -> String {
    let path = match cmd {
        // FIXUP: I do not like this at all.
        Command::Init { path, .. } => path.clone(),
        Command::Pack { path } => path.clone(),
        Command::Publish { path } => path.clone(),
    };
    path.unwrap_or(".".to_string())
}

fn get_cmd_scope(cmd: &Command) -> Option<String> {
    match cmd {
        Command::Init { scope, .. } => scope.clone(),
        _ => None,
    }
}
