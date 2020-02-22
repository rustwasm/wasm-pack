//! Functionality related to running `cargo-generate`.

use child;
use emoji;
use failure::{self, ResultExt};
use std::path::Path;
use std::process::Command;

/// Run `cargo generate` in the current directory to create a new
/// project from a template
pub fn generate(template: &str, name: &str, exec: &Path) -> Result<(), failure::Error> {
    let mut cmd = Command::new(&exec);
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
