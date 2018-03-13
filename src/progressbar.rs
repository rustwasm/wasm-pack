use indicatif::{ProgressBar, ProgressStyle};

pub fn new(msg: String) -> ProgressBar {
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
