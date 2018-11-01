extern crate binary_install;
extern crate curl;
extern crate failure;

use binary_install::{error::Error, install_binaries_from_targz_at_url};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::thread;

const SERVER_URL: &'static str = "localhost:7878";

fn start_server() -> thread::JoinHandle<TcpListener> {
    thread::spawn(|| {
        let listener = TcpListener::bind(SERVER_URL).unwrap();
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();

            let mut buffer = [0; 512];

            stream.read(&mut buffer).unwrap();

            let response = "HTTP/1.1 200 OK\r\n\r\n";

            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        listener
    })
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
    let crate_path = Path::new("");
    let url = format!("http://{}", SERVER_URL);
    let binaries = vec![""];

    // Spin up a local TcpListener.
    start_server();

    let result = install_binaries_from_targz_at_url(crate_path, &url, binaries);
    assert!(result.is_err());

    let err = result.err().unwrap();
    let err = err.downcast_ref::<Error>().unwrap();

    let expected_message = format!("Invalid tarball at {}", url);

    match err {
        Error::Archive { message } => assert_eq!(&expected_message, message),
        _ => panic!("Wrong error returned"),
    }
}
