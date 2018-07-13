//! Functionality related to publishing to npm.

use error::Error;
use std::process::{Command, Stdio};

/// The default npm registry used when we aren't working with a custom registry.
pub const DEFAULT_NPM_REGISTRY: &'static str = "https://registry.npmjs.org/";

/// Run the `npm pack` command.
pub fn npm_pack(path: &str) -> Result<(), Error> {
    let pkg_file_path = format!("{}/pkg", path);
    let output = Command::new("npm")
        .current_dir(pkg_file_path)
        .arg("pack")
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("Packaging up your code failed", s)
    } else {
        Ok(())
    }
}

/// Run the `npm publish` command.
pub fn npm_publish(path: &str) -> Result<(), Error> {
    let pkg_file_path = format!("{}/pkg", path);
    let output = Command::new("npm")
        .current_dir(pkg_file_path)
        .arg("publish")
        .output()?;
    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli("Publishing to npm failed", s)
    } else {
        Ok(())
    }
}

/// Run the `npm login` command.
pub fn npm_login(
    registry: &String,
    scope: &Option<String>,
    always_auth: bool,
    auth_type: &Option<String>,
) -> Result<(), Error> {
    let mut args = String::new();

    args.push_str(&format!("--registry={}", registry));

    if let Some(scope) = scope {
        args.push_str(&format!(" --scope={}", scope));
    }

    if always_auth == true {
        args.push_str(" --always_auth");
    }

    if let Some(auth_type) = auth_type {
        args.push_str(&format!(" --auth_type={}", auth_type));
    }

    let output = Command::new("npm")
        .arg("login")
        .arg(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()?;

    if !output.status.success() {
        let s = String::from_utf8_lossy(&output.stderr);
        Error::cli(&format!("Login to registry {} failed", registry), s)
    } else {
        Ok(())
    }
}
