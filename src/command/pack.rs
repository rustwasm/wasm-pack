use command::utils::{find_pkg_directory, set_crate_path};
use error::Error;
use npm;
use slog::Logger;
use std::path::PathBuf;
use std::result;
use PBAR;

/// Executes the 'npm pack' command on the 'pkg' directory
/// which creates a tarball that can be published to the NPM registry
pub fn pack(path: Option<PathBuf>, log: &Logger) -> result::Result<(), failure::Error> {
    let crate_path = set_crate_path(path)?;

    info!(&log, "Packing up the npm package...");
    let pkg_directory = find_pkg_directory(&crate_path).ok_or(Error::PkgNotFound {
        message: format!(
            "Unable to find the pkg directory at path {:#?}, or in a child directory of {:#?}",
            &crate_path, &crate_path
        ),
    })?;
    npm::npm_pack(log, &pkg_directory.to_string_lossy())?;
    info!(
        &log,
        "Your package is located at {:#?}",
        crate_path.join("pkg")
    );

    PBAR.message("🎒  packed up your package!");
    Ok(())
}
