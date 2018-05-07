use console::style;
use emoji;
use error::Error;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub struct ProgressOutput {
    bar: MultiProgress,
}

impl ProgressOutput {
    pub fn new() -> Self {
        Self {
            bar: MultiProgress::new(),
        }
    }

    pub fn message(&self, message: &str) -> ProgressBar {
        self.bar.add(Self::progressbar(message))
    }

    pub fn info(&self, message: &str) {
        let info = format!(
            "{} {}: {}",
            emoji::INFO,
            style("[INFO]").bold().dim(),
            message
        );
        let bar = self.bar.add(Self::progressbar(&info));
        bar.finish();
    }

    pub fn warn(&self, message: &str) {
        let warn = format!(
            "{} {}: {}",
            emoji::WARN,
            style("[WARN]").bold().dim(),
            message
        );
        let bar = self.bar.add(Self::progressbar(&warn));
        bar.finish();
    }

    pub fn error(&self, message: String) {
        let err = format!(
            "{} {}: {}",
            emoji::ERROR,
            style("[ERR]").bold().dim(),
            message
        );
        let bar = self.bar.add(Self::progressbar(&err));
        bar.finish();
    }

    fn progressbar(msg: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(200);
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("/|\\- ")
                .template("{spinner:.dim.bold} {wide_msg}"),
        );
        pb.set_message(&msg);
        pb
    }

    pub fn done(&self) -> Result<(), Error> {
        self.bar.join_and_clear().map_err(|e| Error::from(e))
    }
}
