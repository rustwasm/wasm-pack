use console::style;
use emoji;
use std::sync::RwLock;
use indicatif::{ProgressBar, ProgressStyle};

pub struct ProgressOutput {
    spinner: RwLock<ProgressBar>,
    messages: RwLock<String>,
}

impl ProgressOutput {
    pub fn new() -> Self {
        Self {
            spinner: RwLock::new(ProgressBar::new_spinner()),
            messages: RwLock::new(String::from("")),
        }
    }

    fn finish(&self) {
        let spinner = self.spinner.read().unwrap();
        spinner.finish();

        let mut message = self.messages.write().unwrap();
        print!("{}", message);
        message.clear();
    }

    pub fn message(&self, message: &str) {
        self.finish();

        let mut spinner = self.spinner.write().unwrap();
        *spinner = Self::progressbar(message);
    }

    fn add_message(&self, msg: &str) {
        let mut message = self.messages.write().unwrap();
        message.push_str("  ");
        message.push_str(msg);
        message.push('\n');
    }

    pub fn info(&self, message: &str) {
        let info = format!(
            "{} {}: {}",
            emoji::INFO,
            style("[INFO]").bold().dim(),
            message
        );
        self.add_message(&info);
    }

    pub fn warn(&self, message: &str) {
        let warn = format!(
            "{} {}: {}",
            emoji::WARN,
            style("[WARN]").bold().dim(),
            message
        );
        self.add_message(&warn);
    }

    pub fn error(&self, message: String) {
        let err = format!(
            "{} {}: {}",
            emoji::ERROR,
            style("[ERR]").bold().dim(),
            message
        );
        self.add_message(&err);
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

    pub fn done(&self) {
        self.finish();
    }
}

impl Drop for ProgressOutput {
    fn drop(&mut self) {
        self.done();
    }
}