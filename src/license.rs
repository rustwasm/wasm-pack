//! Copy `LICENSE` file(s) for the packaged wasm.

use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::manifest::CrateData;
use crate::PBAR;
use glob::glob;

fn glob_license_files(path: &Path) -> Result<impl Iterator<Item = Result<String>> + '_> {
    let joined_path = path.join("LICENSE*");
    let path_string = match joined_path.to_str() {
        Some(path_string) => path_string,
        None => {
            return Err(anyhow!("Could not convert joined license path to String"));
        }
    };

    Ok(glob(path_string)?.map(|entry| match entry {
        Ok(globed_path) => {
            let file_name = match globed_path.file_name() {
                Some(file_name) => file_name,
                None => return Err(anyhow!("Could not get file name from path")),
            };
            let file_name_string = match file_name.to_str() {
                Some(file_name_string) => file_name_string.to_owned(),
                None => return Err(anyhow!("Could not convert filename to String")),
            };
            Ok(file_name_string)
        }
        Err(e) => Err(anyhow!("{:?}", e)),
    }))
}

/// Copy the crate's license into the `pkg` directory.
pub fn copy_from_crate(crate_data: &CrateData, path: &Path, out_dir: &Path) -> Result<()> {
    assert!(
        fs::metadata(path).ok().map_or(false, |m| m.is_dir()),
        "crate directory should exist"
    );

    assert!(
        fs::metadata(&out_dir).ok().map_or(false, |m| m.is_dir()),
        "crate's pkg directory should exist"
    );

    match (crate_data.crate_license(), crate_data.crate_license_file()) {
        (Some(_), _) => {
            let license_files = glob_license_files(path);

            match license_files {
                Ok(files) => {
                    let mut files = files.peekable();
                    if files.peek().is_none() {
                        PBAR.info("License key is set in Cargo.toml but no LICENSE file(s) were found; Please add the LICENSE file(s) to your project directory");
                        return Ok(());
                    }
                    for license_file in files {
                        let license_file = license_file?;
                        let crate_license_path = path.join(&license_file);
                        let new_license_path = out_dir.join(&license_file);
                        if fs::copy(&crate_license_path, &new_license_path).is_err() {
                            PBAR.info("origin crate has no LICENSE");
                        }
                    }
                }
                Err(_) => PBAR.info("origin crate has no LICENSE"),
            }
        }
        (None, Some(license_file)) => {
            let crate_license_path = path.join(&license_file);
            let new_license_path = out_dir.join(&license_file);
            if fs::copy(&crate_license_path, &new_license_path).is_err() {
                PBAR.info("origin crate has no LICENSE");
            }
        }
        (None, None) => {}
    };

    Ok(())
}
