use log::info;
use npm;
use std::result;
use PBAR;

pub fn login(
    registry: Option<String>,
    scope: &Option<String>,
    always_auth: bool,
    auth_type: &Option<String>,
) -> result::Result<(), failure::Error> {
    let registry = registry.unwrap_or_else(|| npm::DEFAULT_NPM_REGISTRY.to_string());

    info!("Logging in to npm...");
    info!(
        "Scope: {:?} Registry: {}, Always Auth: {}, Auth Type: {:?}.",
        &scope, &registry, always_auth, &auth_type
    );
    info!("npm info located in the npm debug log");
    npm::npm_login(&registry, &scope, always_auth, &auth_type)?;
    info!("Logged you in!");

    PBAR.info(&"👋  logged you in!".to_string());
    Ok(())
}
