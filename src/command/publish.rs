use command::utils::set_crate_path;
use error::Error;
use npm;
use slog::Logger;
use std::result;
use PBAR;

pub fn publish(path: Option<String>, log: &Logger) -> result::Result<(), Error> {
    let crate_path = set_crate_path(path);

    info!(&log, "Publishing the npm package...");
    info!(&log, "npm info located in the npm debug log");
    npm::npm_publish(&crate_path).map_err(|e| {
        match e {
        Error::Io { .. } => Error::PkgNotFound {
            message: format!("Unable to find the pkg directory at path '{}', set the path as the parent directory of the pkg directory", &crate_path),
        },
        e => e,
    }
    })?;
    info!(&log, "Published your package!");

    PBAR.message("💥  published your package!");
    Ok(())
}
