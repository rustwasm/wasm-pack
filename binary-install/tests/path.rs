extern crate binary_install;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use binary_install::path::{bin_path, ensure_local_bin_dir, local_bin_path};
use slog::Drain;
use std::path::Path;

fn logger() -> slog::Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    slog::Logger::root(drain, o!())
}

#[test]
fn it_should_get_local_bin_path() {
    let crate_path = Path::new("");

    let expected_path = Path::new("bin/wasm-bindgen");

    let result = local_bin_path(crate_path, "wasm-bindgen");

    assert_eq!(expected_path, result);
}

#[test]
#[cfg(target_os = "windows")]
fn it_should_get_local_bin_path_with_exe_for_windows() {
    let crate_path = Path::new("");

    let expected_path = Path::new("bin/wasm-bindgen.exe");

    let result = local_bin_path(crate_path, "wasm-bindgen");

    assert_eq!(expected_path, result);
}

#[test]
fn it_should_ensure_local_bin_dir_returns_ok_for_folder_that_exists() {
    let crate_path = Path::new("random_folder");

    let result = ensure_local_bin_dir(crate_path);

    assert!(result.is_ok());
}

#[test]
fn it_should_return_some_for_bin_path_that_exists() {
    let crate_path = Path::new("/usr/bin");
    let bin = "ls";

    let result = bin_path(&logger(), crate_path, bin);
    let expected_bin = Path::new("/usr/bin/ls");

    assert!(result.is_some());
    assert_eq!(result.unwrap(), expected_bin);
}

#[test]
fn it_should_return_none_for_bin_path_that_does_not_exists() {
    let crate_path = Path::new("random_folder");
    let bin = "wasm-bindgen";

    let result = bin_path(&logger(), crate_path, bin);

    assert!(result.is_none());
}
