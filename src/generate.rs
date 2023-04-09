//! Functionality related to running `cargo-generate`.

use crate::child;
use crate::emoji;
use crate::install::{self, Tool};
use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::process::Command;

/// Run `cargo generate` in the current directory to create a new
/// project from a template
pub fn generate(template: &OsStr, name: &OsStr, install_status: &install::Status) -> Result<()> {
    let bin_path = install::get_tool_path(install_status, Tool::CargoGenerate)?
        .binary(Tool::CargoGenerate.name())?;
    let mut cmd = Command::new(bin_path);
    cmd.arg("generate");
    cmd.arg("--git").arg(template);
    cmd.arg("--name").arg(name);

    println!(
        "{} Generating a new rustwasm project with name '{}'...",
        emoji::SHEEP,
        name.to_string_lossy()
    );
    child::run(cmd, "cargo-generate").context("Running cargo-generate")?;
    Ok(())
}
