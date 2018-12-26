/// Data structure to represent published package access level.
pub mod access;

use self::access::Access;
use command::build::{Build, BuildOptions};
use command::utils::{find_pkg_directory, set_crate_path};
use dialoguer::{Confirmation, Input};
use failure::Error;
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
) -> result::Result<(), Error> {
    let crate_path = set_crate_path(path)?;

    info!(&log, "Publishing the npm package...");
    info!(&log, "npm info located in the npm debug log");

    let pkg_directory = match find_pkg_directory(&crate_path) {
        Some(path) => Ok(path),
        None => {
            // while `wasm-pack publish`, if the pkg directory cannot be found,
            // then try to `wasm-pack build`
            if Confirmation::new()
                .with_text("Your npm package hasn't be built, build it?")
                .interact()?
            {
                let out_dir = Input::new()
                    .with_prompt("out_dir")
                    .default("pkg".to_string())
                    .show_default(true)
                    .interact()?;
                let target = Input::new()
                    .with_prompt("target")
                    .default("browser".to_string())
                    .show_default(true)
                    .interact()?
                    .to_string();
                let build_opts = BuildOptions {
                    path: Some(crate_path.clone()),
                    target,
                    out_dir: out_dir.clone(),
                    ..Default::default()
                };
                Build::try_from_opts(build_opts)
                    .and_then(|mut build| build.run(&log))
                    .map(|()| crate_path.join(out_dir))
                    .map_err(|_| {
                        format_err!(
                            "Unable to find the pkg directory at path '{:#?}',\
                             or in a child directory of '{:#?}'",
                            &crate_path,
                            &crate_path
                        )
                    })
            } else {
                bail!(
                    "Unable to find the pkg directory at path '{:#?}',\
                     or in a child directory of '{:#?}'",
                    &crate_path,
                    &crate_path
                )
            }
        }
    }?;
    npm::npm_publish(log, &pkg_directory.to_string_lossy(), access)?;
    info!(&log, "Published your package!");

    PBAR.message("💥  published your package!");
    Ok(())
}
