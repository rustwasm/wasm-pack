use command::utils::{find_pkg_directory, get_crate_path};
use failure::Error;
use log::info;
use npm;
use std::path::PathBuf;
use std::result;
use PBAR;

/// Executes the 'npm pack' command on the 'pkg' directory
/// which creates a tarball that can be published to the NPM registry
pub fn pack(path: Option<PathBuf>) -> result::Result<(), Error> {
    let crate_path = get_crate_path(path)?;

    info!("Packing up the npm package...");
    let pkg_directory = find_pkg_directory(&crate_path).ok_or_else(|| {
        format_err!(
            "Unable to find the pkg directory at path {:#?}, or in a child directory of {:#?}",
            &crate_path,
            &crate_path
        )
    })?;
    npm::npm_pack(&pkg_directory.to_string_lossy())?;
    info!("Your package is located at {:#?}", crate_path.join("pkg"));

    PBAR.info("🎒  packed up your package!");
    Ok(())
}
