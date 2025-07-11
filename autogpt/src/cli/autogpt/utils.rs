use anyhow::Context;
use anyhow::Result;
use console::Style;
use indicatif::{ProgressBar, ProgressStyle};
use std::io;
use std::io::Write;
use std::process::Command;
use std::time::Duration;

pub fn success(msg: &str) {
    let green = Style::new().green().bold();
    println!("{}", green.apply_to(msg));
}

pub fn spinner<F: FnOnce() -> Result<()>>(message: &str, func: F) -> Result<()> {
    clear_terminal()?;

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["◑", "◒", "◐", "◓"])
            .template("{spinner} {msg}")?,
    );
    pb.enable_steady_tick(Duration::from_millis(120));
    pb.set_message(message.to_string());

    let result = func();

    pb.finish();

    result
}

fn clear_terminal() -> Result<()> {
    if cfg!(windows) {
        Command::new("cmd")
            .args(["/C", "cls"])
            .status()
            .context("Failed to clear terminal")?;
    } else {
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().context("Failed to flush stdout")?;
    }

    Ok(())
}
