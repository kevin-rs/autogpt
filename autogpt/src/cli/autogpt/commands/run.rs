use crate::cli::autogpt::utils::*;
use anyhow::Result;
use std::process::Command;

pub fn handle_run(feature: String) -> Result<()> {
    spinner("Running application", || {
        Command::new("cargo")
            .arg("run")
            .arg("--features")
            .arg(feature)
            .status()
            .map(|_| ())
            .map_err(|e| e.into())
    })?;

    Ok(())
}
