//! Functionality related to publishing to npm.

use crate::child;
use crate::command::publish::access::Access;
use anyhow::{bail, Context, Result};
use log::info;

/// The default npm registry used when we aren't working with a custom registry.
pub const DEFAULT_NPM_REGISTRY: &str = "https://registry.npmjs.org/";

/// Run the `npm pack` command.
pub fn npm_pack(path: &str) -> Result<()> {
    let mut cmd = child::new_command("npm");
    cmd.current_dir(path).arg("pack");
    child::run(cmd, "npm pack").context("Packaging up your code failed")?;
    Ok(())
}

/// Run the `npm publish` command.
pub fn npm_publish(path: &str, access: Option<Access>, tag: Option<String>) -> Result<()> {
    let mut cmd = child::new_command("npm");
    match access {
        Some(a) => cmd.current_dir(path).arg("publish").arg(&a.to_string()),
        None => cmd.current_dir(path).arg("publish"),
    };
    if let Some(tag) = tag {
        cmd.arg("--tag").arg(tag);
    };

    child::run(cmd, "npm publish").context("Publishing to npm failed")?;
    Ok(())
}

/// Run the `npm login` command.
pub fn npm_login(registry: &str, scope: &Option<String>, auth_type: &Option<String>) -> Result<()> {
    let mut args = vec!["login".to_string(), format!("--registry={}", registry)];

    if let Some(scope) = scope {
        args.push(format!("--scope={}", scope));
    }

    if let Some(auth_type) = auth_type {
        args.push(format!("--auth_type={}", auth_type));
    }

    // Interactively ask user for npm login info.
    //  (child::run does not support interactive input)
    let mut cmd = child::new_command("npm");
    cmd.args(args);

    info!("Running {:?}", cmd);
    if cmd.status()?.success() {
        Ok(())
    } else {
        bail!("Login to registry {} failed", registry)
    }
}
