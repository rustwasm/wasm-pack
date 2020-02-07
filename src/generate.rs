//! Functionality related to running `cargo-generate`.

use child;
use emoji;
use failure::{self, ResultExt};
use install::{self, Tool};
use std::process::Command;

/// Run `cargo generate` in the current directory to create a new
/// project from a template
pub fn generate(
    template: &str,
    name: &str,
    install_status: &install::Status,
) -> Result<(), failure::Error> {
    let bin_path = install::get_tool_path(install_status, Tool::CargoGenerate)?
        .binary(&Tool::CargoGenerate.to_string())?;
    let mut cmd = Command::new(&bin_path);
    cmd.arg("generate");
    cmd.arg("--git").arg(&template);
    cmd.arg("--name").arg(&name);

    println!(
        "{} Generating a new rustwasm project with name '{}'...",
        emoji::SHEEP,
        name
    );
    child::run(cmd, "cargo-generate").context("Running cargo-generate")?;
    Ok(())
}
