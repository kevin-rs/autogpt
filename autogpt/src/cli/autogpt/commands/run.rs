use crate::cli::autogpt::utils::*;
use anyhow::{Result, bail};
use std::env;
use std::process::Command;

pub fn handle_run(mut feature: String) -> Result<()> {
    if feature.trim().is_empty() || feature.trim().eq_ignore_ascii_case("none") {
        let ai_provider = env::var("AI_PROVIDER")
            .map(|v| v.to_lowercase())
            .unwrap_or_else(|_| "unknown".into());

        feature = match ai_provider.as_str() {
            "gemini" => "gem".to_string(),
            "openai" => "oai".to_string(),
            "anthropic" => "cld".to_string(),
            "xai" => "xai".to_string(),
            _ => bail!(
                "❌ Unknown or missing AI_PROVIDER environment variable.\nPlease set AI_PROVIDER to one of: gemini, openai, anthropic, xai."
            ),
        };
    }

    spinner("🚀 Running application", || {
        Command::new("cargo")
            .arg("run")
            .arg("--features")
            .arg(&feature)
            .status()
            .map(|_| ())
            .map_err(|e| e.into())
    })?;

    Ok(())
}
