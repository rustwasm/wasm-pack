//! Functionality related to publishing to npm.

use std::ffi::{OsStr, OsString};

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
    cmd.current_dir(path);
    let arg1 = OsStr::new("publish");
    let access_string = access.map(|a| a.to_string());
    // Using cmd.args(match ...) would cause a "temporary value dropped while borrowed" error on each array
    match (access_string, tag) {
        (None, None) => cmd.args([arg1].as_slice()),
        (Some(arg2), None) | (None, Some(arg2)) => cmd.args([arg1, arg2.as_ref()].as_slice()),
        (Some(arg2), Some(arg3)) => cmd.args([arg1, arg2.as_ref(), arg3.as_ref()].as_slice()),
    };

    child::run(cmd, "npm publish").context("Publishing to npm failed")?;
    Ok(())
}

/// Run the `npm login` command.
pub fn npm_login(registry: &str, scope: &Option<String>, auth_type: &Option<String>) -> Result<()> {
    let arg1 = OsStr::new("login");
    let arg2 = OsString::from(format!("--registry={}", registry));
    let scope_arg = scope.as_deref().map(|s| OsString::from(format!("--scope={}", s)));
    let auth_type_arg = auth_type.as_deref().map(|s| OsString::from(format!("--auth_type={}", s)));
    /*let mut args = vec!["login".to_string(), format!("--registry={}", registry)];

    if let Some(scope) = scope {
        args.push(format!("--scope={}", scope));
    }

    if let Some(auth_type) = auth_type {
        args.push(format!("--auth_type={}", auth_type));
    }*/

    // Interactively ask user for npm login info.
    //  (child::run does not support interactive input)
    let mut cmd = child::new_command("npm");
    match (scope_arg, auth_type_arg) {
        (None, None) => cmd.args([arg1, arg2.as_ref()].as_slice()),
        (Some(arg3), None) | (None, Some(arg3)) => cmd.args([arg1, &arg2, &arg3].as_slice()),
        (Some(arg3), Some(arg4)) => cmd.args([arg1, &arg2, &arg3, &arg4].as_slice()),
    };

    info!("Running {:?}", cmd);
    if cmd.status()?.success() {
        Ok(())
    } else {
        bail!("Login to registry {} failed", registry)
    }
}
