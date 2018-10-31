//! Copy `LICENSE` file(s) for the packaged wasm.

use failure;
use std::fs;
use std::path::Path;

use emoji;
use glob::glob;
use manifest;
use progressbar::Step;
use PBAR;

fn get_license(path: &Path) -> Option<String> {
    match manifest::get_crate_license(path) {
        Ok(license) => license,
        Err(_) => None,
    }
}

fn glob_license_files(path: &Path) -> Result<Vec<String>, failure::Error> {
    let mut license_files: Vec<String> = Vec::new();
    for entry in glob(path.join("LICENSE*").to_str().unwrap())? {
        match entry {
            Ok(globed_path) => {
                license_files.push(String::from(
                    globed_path.file_name().unwrap().to_str().unwrap(),
                ));
            }
            Err(e) => println!("{:?}", e),
        }
    }
    Ok(license_files)
}

/// Copy the crate's license into the `pkg` directory.
pub fn copy_from_crate(path: &Path, out_dir: &Path, step: &Step) -> Result<(), failure::Error> {
    assert!(
        fs::metadata(path).ok().map_or(false, |m| m.is_dir()),
        "crate directory should exist"
    );

    assert!(
        fs::metadata(&out_dir).ok().map_or(false, |m| m.is_dir()),
        "crate's pkg directory should exist"
    );

    match get_license(path) {
        Some(_) => {
            let msg = format!("{}Copying over your LICENSE...", emoji::DANCERS);
            PBAR.step(step, &msg);
            let license_files = glob_license_files(path);

            match license_files {
                Ok(files) => {
                    if files.len() == 0 {
                        PBAR.info("License key is set in Cargo.toml but no LICENSE file(s) were found; Please add the LICENSE file(s) to your project directory");
                        return Ok(());
                    }
                    for license_file in files {
                        let crate_license_path = path.join(&license_file);
                        let new_license_path = out_dir.join(&license_file);
                        if let Err(_) = fs::copy(&crate_license_path, &new_license_path) {
                            PBAR.info("origin crate has no LICENSE");
                        }
                    }
                }
                Err(_) => PBAR.info("origin crate has no LICENSE"),
            }
        }
        None => {
            PBAR.step(step, "No LICENSE found in Cargo.toml, skipping...");
        }
    };

    Ok(())
}
