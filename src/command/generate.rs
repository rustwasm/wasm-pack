use failure::Error;
use generate;
use log::info;
use std::path::Path;
use std::result;
use PBAR;

/// Executes the 'cargo-generate' command in the current directory
/// which generates a new rustwasm project from a template.
pub fn generate(template: String, name: &str, exec_path: &Path) -> result::Result<(), Error> {
    info!("Generating a new rustwasm project...");
    generate::generate(&template, &name, exec_path)?;

    let msg = format!("🐑 Generated new project at /{}", name);
    PBAR.info(&msg);
    Ok(())
}
