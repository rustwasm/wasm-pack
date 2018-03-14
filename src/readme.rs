use std::fs;
use failure::Error;
use console::{style, Emoji};
use indicatif::MultiProgress;

use progressbar;

static WARN: Emoji = Emoji("⚠️  ", ":-)");
static DANCERS: Emoji = Emoji("👯  ", "");

pub fn copy_from_crate(path: &str) -> Result<(), Error> {
    let m = MultiProgress::new();
    let step = format!(
            "{} {}Copying over your README...",
            style("[5/7]").bold().dim(),
            DANCERS
    );
    let pb = m.add(progressbar::new(step));
    let crate_readme_path = format!("{}/README.md", path);
    let new_readme_path = format!("{}/pkg/README.md", path);
    if let Err(_) = fs::copy(&crate_readme_path, &new_readme_path) {
        let warn = format!("{} {}: origin crate has no README", WARN, style("[WARN]").bold().dim());
        let warn_pb = m.add(progressbar::new(warn));
        warn_pb.finish();
    };
    pb.finish();
    m.join()?;
    Ok(())
}
