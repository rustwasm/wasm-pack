use npm;
use slog::Logger;
use std::result;
use PBAR;

pub fn login(
    registry: Option<String>,
    scope: Option<String>,
    always_auth: bool,
    auth_type: Option<String>,
    log: &Logger,
) -> result::Result<(), failure::Error> {
    let registry = registry.unwrap_or(npm::DEFAULT_NPM_REGISTRY.to_string());

    info!(&log, "Logging in to npm...");
    info!(
        &log,
        "Scope: {:?} Registry: {}, Always Auth: {}, Auth Type: {:?}.",
        &scope,
        &registry,
        always_auth,
        &auth_type
    );
    info!(&log, "npm info located in the npm debug log");
    npm::npm_login(log, &registry, &scope, always_auth, &auth_type)?;
    info!(&log, "Logged you in!");

    PBAR.message(&format!("👋  logged you in!"));
    Ok(())
}
