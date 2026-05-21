// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::cli::autogpt::ast::AgentConfig;
use crate::cli::autogpt::utils::*;
use anyhow::{Context, Result, bail};
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, process::Command};
use toml_edit::{Array, DocumentMut, InlineTable, Item, Table, Value};

pub fn handle_new(name: &str, feature: Option<String>) -> Result<()> {
    let path = Path::new(name);
    if path.exists() {
        bail!("❌ Directory '{}' already exists", name);
    }

    let (provider, model, feat_flag) = match feature.as_deref() {
        Some("hf") => ("huggingface", "meta-llama/Llama-3.3-70B-Instruct", "hf"),
        Some("oai") => ("openai", "gpt-5", "oai"),
        Some("cld") => ("anthropic", "claude-opus-4-6", "cld"),
        Some("xai") => ("xai", "grok-4", "xai"),
        Some("co") => ("cohere", "command-a-03-2025", "co"),
        Some("gem") | None => ("gemini", "gemini-3.0-flash", "gem"),
        Some(unknown) => bail!(
            "❌ Unknown feature '{}'. Supported features are: gem, hf, oai, cld, xai, co.",
            unknown
        ),
    };

    let yaml_content = format!(
        r#"name: {name}
ai_provider: {provider}
model: {model}
persona: Backend Engineer
role: user
prompt: |
  Describe a scalable microservice architecture.
"#
    );

    spinner("📦 Scaffolding project", || {
        Command::new("cargo")
            .arg("new")
            .arg("--bin")
            .arg(name)
            .status()
            .context("Failed to create cargo project")?;

        fs::write(path.join("agent.yaml"), yaml_content.trim_start())?;
        fs::write(
            path.join("README.md"),
            format!("# {name}\n\nCreated with `autogpt new`."),
        )?;

        let cargo_toml_path = path.join("Cargo.toml");
        let mut doc: DocumentMut = fs::read_to_string(&cargo_toml_path)?
            .parse()
            .context("Failed to parse Cargo.toml")?;

        let deps = doc["dependencies"].or_insert(Item::Table(Table::new()));
        let deps = deps.as_table_mut().unwrap();

        let mut tokio_table = InlineTable::new();
        tokio_table.insert("version", Value::from("1.52.2"));
        tokio_table.insert(
            "features",
            Value::Array({
                let mut a = Array::default();
                a.push("full");
                a
            }),
        );
        deps.insert("tokio", Item::Value(Value::InlineTable(tokio_table)));

        let mut autogpt_table = InlineTable::new();
        autogpt_table.insert("version", Value::from(env!("CARGO_PKG_VERSION")));
        autogpt_table.insert("default-features", Value::from(false));
        autogpt_table.insert(
            "features",
            Value::Array({
                let mut a = Array::default();
                a.push(feat_flag);
                a
            }),
        );
        deps.insert("autogpt", Item::Value(Value::InlineTable(autogpt_table)));

        let features = doc["features"].or_insert(Item::Table(Table::new()));
        let f_table = features.as_table_mut().unwrap();

        for feat in [
            "gem", "mcp", "net", "mem", "oai", "cld", "xai", "co", "mta", "hf", "col",
        ] {
            let mut arr = Array::default();
            if feat == feat_flag {
                arr.push(format!("autogpt/{}", feat));
            }
            f_table.insert(feat, Item::Value(Value::Array(arr)));
        }

        fs::write(&cargo_toml_path, doc.to_string())?;

        Ok(())
    })?;

    set_env(path)?;

    success(&format!("✅ Project created at ./{name}"));
    Ok(())
}

fn set_env(project_path: &Path) -> Result<()> {
    let yaml_path = project_path.join("agent.yaml");
    let yaml_content = fs::read_to_string(&yaml_path).context("Failed to read agent.yaml")?;

    let config: AgentConfig =
        serde_yaml::from_str(&yaml_content).context("Failed to parse agent.yaml")?;

    if cfg!(windows) {
        set_env_win("AI_PROVIDER", &config.ai_provider)?;
        if config.ai_provider == "gemini" {
            set_env_win("GEMINI_MODEL", &config.model)?;
        }
        success("✅ Environment variables set using 'setx'.\nRestart your terminal to apply them.");
    } else {
        let profile_path = find_shell()?;
        append(&profile_path, "AI_PROVIDER", &config.ai_provider)?;
        if config.ai_provider == "gemini" {
            append(&profile_path, "GEMINI_MODEL", &config.model)?;
        }
        success(&format!(
            "✅ Environment variables added to '{}'.\nRun `source {}` or restart your terminal to apply them.",
            profile_path.display(),
            profile_path.display()
        ));
    }

    Ok(())
}

fn set_env_win(key: &str, value: &str) -> Result<()> {
    let status = Command::new("setx")
        .arg(key)
        .arg(value)
        .status()
        .context("Failed to execute 'setx'")?;

    if !status.success() {
        bail!(
            "setx failed with exit code: {}",
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

fn append(profile_path: &Path, key: &str, value: &str) -> Result<()> {
    let export_line = format!("export {key}={value}\n");
    let existing = fs::read_to_string(profile_path).unwrap_or_default();

    if !existing.contains(export_line.trim()) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(profile_path)?;
        file.write_all(export_line.as_bytes())?;
    }

    Ok(())
}

fn find_shell() -> Result<PathBuf> {
    let home = env::var("HOME").context("HOME environment variable not set")?;
    let candidates = [".bashrc", ".zshrc", ".profile"];
    for filename in &candidates {
        let path = Path::new(&home).join(filename);
        if path.exists() {
            return Ok(path);
        }
    }
    Ok(Path::new(&home).join(".bashrc"))
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
