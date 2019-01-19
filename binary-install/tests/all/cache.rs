use binary_install::Cache;
use std::path::Path;
use utils;

#[test]
fn it_returns_none_if_install_is_not_permitted() {
    let binary_name = "wasm-pack";
    let binaries = vec![binary_name];

    let dir = tempfile::TempDir::new().unwrap();
    let cache = Cache::at(dir.path());

    let dl = cache.download(
        false,
        binary_name,
        &binaries,
        &format!("{}/{}.tar.gz", "", binary_name),
    );

    assert!(dl.is_ok());
    assert!(dl.unwrap().is_none())
}

#[test]
fn it_downloads_tarball() {
    let server_port = 7880;
    let url = format!("http://{}:{}", utils::TEST_SERVER_HOST, server_port);
    let binary_name = "wasm-pack";
    let binaries = vec![binary_name];

    // Create a temporary tarball.
    let tarball = utils::create_tarball(binary_name).ok();

    // Spin up a local TcpListener.
    utils::start_server(server_port, tarball);

    let dir = tempfile::TempDir::new().unwrap();
    let cache = Cache::at(dir.path());

    let dl = cache.download(
        true,
        binary_name,
        &binaries,
        &format!("{}/{}.tar.gz", &url, binary_name),
    );

    assert!(dl.is_ok());
    assert!(dl.unwrap().is_some())
}

#[test]
fn it_returns_error_when_it_failed_to_download() {
    let server_port = 7881;
    let url = format!("http://{}:{}", utils::TEST_SERVER_HOST, server_port);
    let binary_name = "wasm-pack";
    let binaries = vec![binary_name];

    let dir = tempfile::TempDir::new().unwrap();
    let cache = Cache::at(dir.path());
    let full_url = &format!("{}/{}.tar.gz", &url, binary_name);

    let dl = cache.download(true, binary_name, &binaries, full_url);

    assert!(dl.is_err());
    assert_eq!(
        &format!("failed to download from {}", full_url),
        &format!("{}", dl.unwrap_err())
    );
}

#[test]
fn it_returns_error_when_it_failed_to_extract_tarball() {
    let server_port = 7882;
    let url = format!("http://{}:{}", utils::TEST_SERVER_HOST, server_port);
    let binary_name = "wasm-pack";
    let binaries = vec![binary_name];

    let dir = tempfile::TempDir::new().unwrap();
    let cache = Cache::at(dir.path());
    let full_url = &format!("{}/{}.tar.gz", &url, binary_name);

    // Spin up a local TcpListener.
    utils::start_server(server_port, None);

    let dl = cache.download(true, binary_name, &binaries, full_url);

    assert!(dl.is_err());
    assert_eq!(
        &format!("failed to extract tarball from {}", full_url),
        &format!("{}", dl.unwrap_err())
    );
}

#[test]
fn it_returns_error_when_it_failed_to_extract_zip() {
    let server_port = 7883;
    let url = format!("http://{}:{}", utils::TEST_SERVER_HOST, server_port);
    let binary_name = "wasm-pack";
    let binaries = vec![binary_name];

    let dir = tempfile::TempDir::new().unwrap();
    let cache = Cache::at(dir.path());
    let full_url = &format!("{}/{}.zip", &url, binary_name);

    // Spin up a local TcpListener.
    utils::start_server(server_port, None);

    let dl = cache.download(true, binary_name, &binaries, full_url);

    assert!(dl.is_err());
    assert_eq!(
        &format!("failed to extract zip from {}", full_url),
        &format!("{}", dl.unwrap_err())
    );
}

#[test]
#[should_panic(expected = "don't know how to extract http://localhost:7884/wasm-pack.bin")]
fn it_panics_if_not_tarball_or_zip() {
    let server_port = 7884;
    let url = format!("http://{}:{}", utils::TEST_SERVER_HOST, server_port);
    let binary_name = "wasm-pack";
    let binaries = vec![binary_name];

    let dir = tempfile::TempDir::new().unwrap();
    let cache = Cache::at(dir.path());
    let full_url = &format!("{}/{}.bin", &url, binary_name);

    // Spin up a local TcpListener.
    utils::start_server(server_port, None);

    let _ = cache.download(true, binary_name, &binaries, full_url);
}

#[test]
fn it_joins_path_with_destination() {
    let dir = tempfile::TempDir::new().unwrap();
    let cache = Cache::at(dir.path());

    assert_eq!(dir.path().join("hello"), cache.join(Path::new("hello")));
}
