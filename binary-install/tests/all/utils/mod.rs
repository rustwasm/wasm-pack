use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

pub const TEST_SERVER_HOST: &'static str = "localhost";

pub fn start_server(tarball: Option<Vec<u8>>, server_port: Option<u16>) -> Receiver<u16> {
    let (sender, receiver) = channel();

    thread::spawn(move || {
        TcpListener::bind(format!(
            "{}:{}",
            TEST_SERVER_HOST,
            server_port.unwrap_or_else(|| 0)
        ))
        .map(|listener| {
            sender.send(listener.local_addr().unwrap().port()).unwrap();

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
        })
        .unwrap();
    });

    receiver
}

pub fn create_tarball(binary_name: &str) -> Result<Vec<u8>, io::Error> {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let full_path = temp_dir.path().join(binary_name.to_owned() + ".tar.gz");

    let tar = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&full_path)?;

    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(temp_dir.path().join(binary_name))?;

    let mut encoder = GzEncoder::new(tar, Compression::default());
    {
        let mut archive = tar::Builder::new(&mut encoder);
        archive.append_file(binary_name, &mut file)?;
    }

    let mut contents = vec![];

    encoder.finish()?;

    File::open(temp_dir.path().join(&full_path))?.read_to_end(&mut contents)?;

    Ok(contents)
}
