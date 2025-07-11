use crate::cli::autogpt::{generator::generate_code, parser::parse_yaml, utils::*};
use anyhow::Result;
use std::path::Path;
use std::process::Stdio;

pub fn handle_build(out: Option<String>) -> Result<()> {
    let config = parse_yaml("agent.yaml")?;
    let output_path = out
        .as_deref()
        .map(Path::new)
        .unwrap_or(Path::new("src/main.rs"));

    spinner("Generating Rust code", || {
        generate_code(&config, output_path)
    })?;

    success("✅ Code generation complete");

    spinner("Compiling project", || {
        std::process::Command::new("cargo")
            .arg("build")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .status()
            .map(|_| ())
            .map_err(|e| e.into())
    })?;

    success("✅ Build complete");
    Ok(())
}
