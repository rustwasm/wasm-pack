extern crate binary_install;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use binary_install::path::{bin_path, ensure_local_bin_dir, local_bin_path};
use slog::Drain;
use std::env::current_dir;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn logger() -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    slog::Logger::root(drain, o!())
}

fn get_tests_bin_path() -> PathBuf {
    let path = current_dir().unwrap();
    path.join("tests")
}

#[test]
#[cfg(not(target_ = "wasm32"))]
fn get_local_bin_path_should_return_a_path() {
    let crate_path = Path::new("");

    let expected_path = Path::new("bin/wasm-bindgen");

    let result = local_bin_path(crate_path, "wasm-bindgen");

    assert_eq!(expected_path, result);
}

#[test]
#[cfg(target_os = "windows")]
fn get_local_bin_path_should_return_with_exe_for_windows() {
    let crate_path = Path::new("");

    let expected_path = Path::new("bin/wasm-bindgen.exe");

    let result = local_bin_path(crate_path, "wasm-bindgen");

    assert_eq!(expected_path, result);
}

#[test]
fn ensure_local_bin_dir_should_return_ok_for_folder_that_exists() {
    let crate_path = get_tests_bin_path().join("bin2");

    fs::create_dir_all(crate_path.to_owned()).unwrap();

    let result = ensure_local_bin_dir(&crate_path);

    assert!(result.is_ok());

    fs::remove_dir_all(crate_path).unwrap();
}

#[test]
fn ensure_local_bin_dir_should_create_folder_if_it_doesnt_exist() {
    let crate_path = get_tests_bin_path().join("bin3");

    // Make sure that the folder doesn't exist
    // before we call ensure_local_bin_dir();
    let dir = fs::read_dir(crate_path.to_owned());
    let dir_error = dir.err().unwrap();
    assert_eq!(dir_error.kind(), io::ErrorKind::NotFound);

    let result = ensure_local_bin_dir(&crate_path);

    assert!(result.is_ok());

    // Make sure that the directory actually exists.
    let dir = fs::read_dir(crate_path.to_owned());
    assert!(dir.is_ok());

    fs::remove_dir_all(crate_path).unwrap();
}

#[test]
fn bin_path_should_return_some_for_path_that_exists() {
    let crate_path = get_tests_bin_path();
    let bin_file = "wasm-bindgen";
    let full_bin_path = crate_path.join("bin");
    let path_with_bin_file = full_bin_path.join(bin_file);

    fs::create_dir_all(full_bin_path.to_owned()).unwrap();
    fs::File::create(path_with_bin_file.to_owned()).unwrap();

    let result = bin_path(&logger(), &crate_path, bin_file);

    assert!(result.is_some());
    assert_eq!(path_with_bin_file, result.unwrap());

    fs::remove_dir_all(full_bin_path).unwrap();
}

#[test]
fn bin_path_should_return_none_for_path_that_does_not_exists() {
    let crate_path = get_tests_bin_path();
    let bin_file = "wasm-binfile";
    let full_bin_path = crate_path.join("bin_path");
    let path_with_bin_file = full_bin_path.join(bin_file);

    // Make sure that the folder doesn't exist
    // before we call bin_path();
    let file = fs::File::open(path_with_bin_file.to_owned());
    let file_error = file.err().unwrap();
    assert_eq!(file_error.kind(), io::ErrorKind::NotFound);

    let result = bin_path(&logger(), &crate_path, bin_file);

    assert!(result.is_none());
}
