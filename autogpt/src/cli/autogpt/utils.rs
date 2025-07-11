use anyhow::Result;
use console::Style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub fn success(msg: &str) {
    let green = Style::new().green().bold();
    println!("{}", green.apply_to(msg));
}

pub fn spinner<F: FnOnce() -> Result<()>>(message: &str, func: F) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["◑", "◒", "◐", "◓", "◑", "✅"])
            .template("{spinner} {msg}")?,
    );
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_message(message.to_string());

    let result = func();

    pb.finish_with_message("Done");
    result
}
