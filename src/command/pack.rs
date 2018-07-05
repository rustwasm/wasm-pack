use command::utils::set_crate_path;
use error::Error;
use npm;
use slog::Logger;
use std::result;
use PBAR;

pub fn pack(path: Option<String>, log: &Logger) -> result::Result<(), Error> {
    let crate_path = set_crate_path(path);

    info!(&log, "Packing up the npm package...");
    match npm::npm_pack(&crate_path) {
        Ok(r) => Ok(r),
        Err(Error::Io { .. }) => Err(Error::DirNotFound {
            message: "Unable to find the pkg directory".to_owned(),
        }),
        Err(e) => Err(e),
    }?;
    #[cfg(not(target_os = "windows"))]
    info!(&log, "Your package is located at {}/pkg", &crate_path);
    #[cfg(target_os = "windows")]
    info!(&log, "Your package is located at {}\\pkg", &crate_path);

    PBAR.message("🎒  packed up your package!");
    Ok(())
}
