use crate::npm;
use crate::PBAR;
use anyhow::Result;
use log::info;

pub fn login(
    registry: Option<String>,
    scope: &Option<String>,
    auth_type: &Option<String>,
) -> Result<()> {
    let registry = registry.unwrap_or_else(|| npm::DEFAULT_NPM_REGISTRY.to_string());

    info!("Logging in to npm...");
    info!(
        "Scope: {:?} Registry: {}, Auth Type: {:?}.",
        &scope, &registry, &auth_type
    );
    info!("npm info located in the npm debug log");
    npm::npm_login(&registry, &scope, &auth_type)?;
    info!("Logged you in!");

    PBAR.info(&"👋  logged you in!".to_string());
    Ok(())
}
