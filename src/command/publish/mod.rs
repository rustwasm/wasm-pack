/// Data structure to represent published package access level.
pub mod access;

use self::access::Access;
use command::utils::{find_pkg_directory, set_crate_path};
use error::Error;
use npm;
use slog::Logger;
use std::path::PathBuf;
use std::result;
use PBAR;

/// Creates a tarball from a 'pkg' directory
/// and publishes it to the NPM registry
pub fn publish(
    path: Option<PathBuf>,
    access: Option<Access>,
    log: &Logger,
) -> result::Result<(), failure::Error> {
    let crate_path = set_crate_path(path)?;

    info!(&log, "Publishing the npm package...");
    info!(&log, "npm info located in the npm debug log");
    let pkg_directory = find_pkg_directory(&crate_path).ok_or(Error::PkgNotFound {
        message: format!(
            "Unable to find the pkg directory at path '{:#?}', or in a child directory of '{:#?}'",
            &crate_path, &crate_path
        ),
    })?;

    npm::npm_publish(log, &pkg_directory.to_string_lossy(), access)?;
    info!(&log, "Published your package!");

    PBAR.message("💥  published your package!");
    Ok(())
}
