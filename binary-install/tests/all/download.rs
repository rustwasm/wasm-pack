use binary_install::Download;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;

#[test]
#[cfg(unix)]
fn it_returns_binary_name() {
    let binary_name = "wasm-pack";

    let dir = tempfile::TempDir::new().unwrap();
    let download = Download::at(dir.path());

    let full_path = dir.path().join(binary_name);

    let mut options = OpenOptions::new();
    options.create(true);
    options.write(true);

    // Make the "binary" an executable.
    options.mode(0o755);

    options.open(&full_path).unwrap();

    let binary = download.binary(binary_name);

    assert!(binary.is_ok());
    assert_eq!(full_path, binary.unwrap());
}

#[test]
fn it_bails_if_not_file() {
    let binary_name = "wasm-pack";

    let dir = tempfile::TempDir::new().unwrap();
    let download = Download::at(dir.path());

    let full_path = dir.path().join(binary_name);

    let mut options = OpenOptions::new();
    options.create(true);
    options.write(true);

    let binary = download.binary(binary_name);

    assert!(binary.is_err());
    assert_eq!(
        format!("{} binary does not exist", full_path.to_str().unwrap()),
        binary.unwrap_err().to_string()
    );
}

#[test]
fn it_bails_if_not_executable() {
    let binary_name = "wasm-pack";

    let dir = tempfile::TempDir::new().unwrap();
    let download = Download::at(dir.path());

    let full_path = dir.path().join(binary_name);

    let mut options = OpenOptions::new();
    options.create(true);
    options.write(true);

    options.open(&full_path).unwrap();

    let binary = download.binary(binary_name);

    assert!(binary.is_err());
    assert_eq!(
        format!("{} is not executable", full_path.to_str().unwrap()),
        binary.unwrap_err().to_string()
    );
}
