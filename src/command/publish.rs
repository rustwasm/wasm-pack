use command::utils::{find_pkg_directory, set_crate_path};
use error::Error;
use npm;
use slog::Logger;
use std::result;
use PBAR;

pub fn publish(path: Option<String>, log: &Logger) -> result::Result<(), Error> {
    let crate_path = set_crate_path(path);

    info!(&log, "Publishing the npm package...");
    info!(&log, "npm info located in the npm debug log");
    let pkg_directory = find_pkg_directory(&crate_path).ok_or(Error::PkgNotFound {
        message: format!(
            "Unable to find the pkg directory at path '{}', or in a child directory of '{}'",
            &crate_path, &crate_path
        ),
    })?;

    npm::npm_publish(&pkg_directory.to_string_lossy())?;
    info!(&log, "Published your package!");

    PBAR.message("💥  published your package!");
    Ok(())
}
