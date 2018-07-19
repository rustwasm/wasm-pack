use command::utils::{find_pkg_directory, set_crate_path};
use error::Error;
use npm;
use slog::Logger;
use std::result;
use PBAR;

/// Executes the 'npm pack' command on the 'pkg' directory
/// which creates a tarball that can be published to the NPM registry
pub fn pack(path: Option<String>, log: &Logger) -> result::Result<(), Error> {
    let crate_path = set_crate_path(path);

    info!(&log, "Packing up the npm package...");
    let pkg_directory = find_pkg_directory(&crate_path).ok_or(Error::PkgNotFound {
        message: format!(
            "Unable to find the pkg directory at path '{}', or in a child directory of '{}'",
            &crate_path, &crate_path
        ),
    })?;
    npm::npm_pack(&pkg_directory.to_string_lossy())?;
    #[cfg(not(target_os = "windows"))]
    info!(&log, "Your package is located at {}/pkg", &crate_path);
    #[cfg(target_os = "windows")]
    info!(&log, "Your package is located at {}\\pkg", &crate_path);

    PBAR.message("🎒  packed up your package!");
    Ok(())
}
