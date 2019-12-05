//! Implementation of the `wasm-pack serve` command.

use super::build::Build;
use super::watch::watch_lock;
use failure::Error;
use hyper::rt::Future;
use hyper::service::service_fn;
use hyper::Server;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread::spawn;

pub fn serve(
    build: Build,
    no_watch: bool,
    port: Option<u16>,
    root: Option<PathBuf>,
) -> Result<(), Error> {
    let build_lock = Arc::new(RwLock::new(()));

    let port = port.unwrap_or(8000);
    let root = root.unwrap_or_else(|| build.crate_path.clone());
    if !no_watch {
        let build_lock = build_lock.clone();
        spawn(move || {
            let _result = watch_lock(build, build_lock);
        });
    }

    // TODO should this be IPv4 or IPv6?
    let addr = ([127, 0, 0, 1], port).into();
    let root = Arc::new(root);
    let server = Server::bind(&addr)
        .serve(move || {
            let build_lock = build_lock.clone();
            let root = root.clone();
            service_fn(move |req| {
                let _guard = build_lock.read();
                let resolve = hyper_staticfile::resolve(&*root, &req);
                resolve.map(move |result| {
                    hyper_staticfile::ResponseBuilder::new()
                        .build(&req, result)
                        .unwrap()
                })
            })
        })
        .map_err(|x| eprintln!("server error: {}", x));

    println!("Serving on http://127.0.0.1:{}", port);

    hyper::rt::run(server);

    Ok(())
}
