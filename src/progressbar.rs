use console::style;
use emoji;
use error::Error;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::RwLock;

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

    fn finish(&self) -> Result<(), Error> {
        let spinner = self.spinner.read()?;
        spinner.finish();

        let mut message = self.messages.write()?;
        print!("{}", message);
        message.clear();

        Ok(())
    }

    pub fn message(&self, message: &str) -> Result<(), Error> {
        self.finish()?;

        let mut spinner = self.spinner.write()?;
        *spinner = Self::progressbar(message);
        Ok(())
    }

    fn add_message(&self, msg: &str) -> Result<(), Error> {
        let mut message = self.messages.write()?;
        message.push_str("  ");
        message.push_str(msg);
        message.push('\n');

        Ok(())
    }

    pub fn info(&self, message: &str) -> Result<(), Error> {
        let info = format!(
            "{} {}: {}",
            emoji::INFO,
            style("[INFO]").bold().dim(),
            message
        );
        self.add_message(&info)?;

        Ok(())
    }

    pub fn warn(&self, message: &str) -> Result<(), Error> {
        let warn = format!(
            "{} {}: {}",
            emoji::WARN,
            style("[WARN]").bold().dim(),
            message
        );
        self.add_message(&warn)?;

        Ok(())
    }

    pub fn error(&self, message: String) -> Result<(), Error> {
        let err = format!(
            "{} {}: {}",
            emoji::ERROR,
            style("[ERR]").bold().dim(),
            message
        );
        self.add_message(&err)?;

        Ok(())
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
        self.finish()?;
        Ok(())
    }
}

impl Drop for ProgressOutput {
    fn drop(&mut self) {
        self.done().ok();
    }
}
