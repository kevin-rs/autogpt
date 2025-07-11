use crate::cli::autogpt::ast::AgentConfig;
use anyhow::{Context, Result, bail};
use std::fs;

pub fn parse_yaml(path: &str) -> Result<AgentConfig> {
    let content = fs::read_to_string(path).context("Failed to read YAML file")?;
    let config: AgentConfig = serde_yaml::from_str(&content).context("Invalid YAML structure")?;

    validate_config(&config)?;
    Ok(config)
}

fn validate_config(config: &AgentConfig) -> Result<()> {
    if config.name.is_empty() {
        bail!("Agent name is required");
    }
    if config.prompt.trim().is_empty() {
        bail!("Prompt cannot be empty");
    }
    Ok(())
}
