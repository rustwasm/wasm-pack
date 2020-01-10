//! Implementation of the `wasm-pack watch` command.

use super::build::Build;
use failure::Error;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, sync_channel};
use std::sync::{Arc, RwLock};
use std::thread::spawn;
use std::time::Duration;

pub fn watch(build: Build) -> Result<(), Error> {
    watch_lock(build, Arc::new(RwLock::new(())))
}

pub fn watch_lock(mut build: Build, build_lock: Arc<RwLock<()>>) -> Result<(), Error> {
    // TODO make this more reasonable

    // start by building
    {
        let _guard = build_lock.write();
        match build.run() {
            Ok(_) => (),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    let watch_path = build.crate_path.join("src");

    let (event_tx, event_rx) = channel();
    let (build_tx, build_rx) = sync_channel(0);

    let mut watcher = watcher(event_tx, Duration::from_millis(200))?;

    watcher.watch(watch_path, RecursiveMode::Recursive)?;

    let halt = Arc::new(AtomicBool::new(false));

    let event_handle = {
        let halt = halt.clone();
        spawn(move || {
            while !halt.load(Ordering::SeqCst) {
                match event_rx.recv() {
                    Ok(DebouncedEvent::NoticeWrite(_)) | Ok(DebouncedEvent::NoticeRemove(_)) => (),
                    Ok(_) => {
                        // if the send fails because there's already a build queued, we do not care
                        let _ = build_tx.try_send(());
                    }
                    _ => (),
                }
            }
        })
    };

    while !halt.load(Ordering::SeqCst) {
        match build_rx.recv() {
            Ok(_) => {
                let _guard = build_lock.write();
                match build.run() {
                    Ok(_) => (),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Err(e) => eprintln!("Watch error: {}", e),
        }
    }
    Ok(())
}
