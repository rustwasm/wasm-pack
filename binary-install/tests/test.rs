extern crate binary_install;
extern crate curl;
extern crate failure;
extern crate flate2;
extern crate tar;

use binary_install::{error::Error, install_binaries_from_targz_at_url};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::thread;

const SERVER_HOST: &'static str = "localhost";

fn start_server(port: u32, tarball: Option<Vec<u8>>) -> thread::JoinHandle<TcpListener> {
    thread::spawn(move || {
        let listener = TcpListener::bind(format!("{}:{}", SERVER_HOST, port)).unwrap();

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();

            let mut buffer = [0; 512];

            stream.read(&mut buffer).unwrap();

            let response = "HTTP/1.1 200 OK\r\n\r\n";

            stream.write(response.as_bytes()).unwrap();

            match tarball.to_owned() {
                Some(tar) => {
                    stream.write(tar.as_ref()).unwrap();
                }
                None => {}
            }

            stream.flush().unwrap();
        }
        listener
    })
}

fn create_tarball(binary_name: &str) -> Result<Vec<u8>, io::Error> {
    let temp_dir = env::temp_dir();
    let tar = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(temp_dir.join("foo.tar.gz"))?;

    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(temp_dir.join(binary_name))?;

    let mut encoder = GzEncoder::new(tar, Compression::default());
    {
        let mut archive = tar::Builder::new(&mut encoder);
        archive.append_file(binary_name, &mut file)?;
    }

    let mut contents = vec![];

    encoder.finish()?;

    File::open(temp_dir.join("foo.tar.gz"))?.read_to_end(&mut contents)?;

    Ok(contents)
}

#[test]
fn install_binaries_from_targz_at_url_should_return_http_error_for_bad_url() {
    let crate_path = Path::new("");
    let url = "";
    let binaries = vec![""];

    let result = install_binaries_from_targz_at_url(crate_path, url, binaries);
    assert!(result.is_err());

    let err = result.err().unwrap();
    let err = err.downcast_ref::<Error>().unwrap();

    let expected_message = format!("when requesting {}", url);

    match err {
        Error::Http { message } => assert_eq!(&expected_message, message),
        _ => panic!("Wrong error returned"),
    }
}

#[test]
fn install_binaries_from_targz_at_url_should_return_archive_error_when_tarball_is_missing() {
    let server_port = 7878;
    let url = format!("http://{}:{}", SERVER_HOST, server_port);
    let crate_path = Path::new("");
    let binaries = vec![""];

    // Spin up a local TcpListener.
    start_server(server_port, None);

    let result = install_binaries_from_targz_at_url(crate_path, &url, binaries);
    assert!(result.is_err());

    let err = result.err().unwrap();
    let err = err.downcast_ref::<Error>().unwrap();

    let expected_message = format!(
        "Invalid tarball at {}. Inner error: failed to fill whole buffer",
        url
    );

    match err {
        Error::Archive { message } => assert_eq!(&expected_message, message),
        _ => panic!("Wrong error returned"),
    }
}

#[test]
fn install_binaries_from_targz_at_url_should_return_archive_error_when_tarball_does_not_include_all_files(
) {
    let server_port = 7879;
    let url = format!("http://{}:{}", SERVER_HOST, server_port);
    let crate_path = Path::new("");
    let binaries = vec!["wasm-pack"];

    // Create a temporary tarball.
    let tarball = create_tarball("foo.txt").ok();
    // Spin up a local TcpListener.
    start_server(server_port, tarball);

    let result = install_binaries_from_targz_at_url(crate_path, &url, binaries);
    assert!(result.is_err());

    let err = result.err().unwrap();
    let err = err.downcast_ref::<Error>().unwrap();

    let expected_message = format!(
        "the tarball at {} was missing expected executables: {}",
        url, "wasm-pack"
    );

    match err {
        Error::Archive { message } => assert_eq!(&expected_message, message),
        _ => panic!("Wrong error returned"),
    }
}

#[test]
fn install_binaries_from_targz_at_url_should_return_ok_if_binary_is_found() {
    let server_port = 7880;
    let url = format!("http://{}:{}", SERVER_HOST, server_port);
    let binary_name = "wasm-pack";
    let crate_path = Path::new("");
    let binaries = vec![binary_name];

    // Create a temporary tarball.
    let tarball = create_tarball(binary_name).ok();
    // Spin up a local TcpListener.
    start_server(server_port, tarball);

    let result = install_binaries_from_targz_at_url(crate_path, &url, binaries);
    assert!(result.is_ok());
}
