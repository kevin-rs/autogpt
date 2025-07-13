use crate::cli::autogpt::ast::AgentConfig;
use crate::cli::autogpt::utils::*;
use anyhow::{Context, Result, bail};
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, process::Command};
use toml_edit::{Array, DocumentMut, InlineTable, Item, Table, Value};

const DEFAULT_YAML: &str = r#"
name: agent_name
ai_provider: gemini
model: gemini-2.0-flash
position: Backend Engineer
role: user
prompt: |
  Describe a scalable microservice architecture.
"#;

pub fn handle_new(name: &str) -> Result<()> {
    let path = Path::new(name);
    if path.exists() {
        bail!("❌ Directory '{}' already exists", name);
    }

    spinner("📦 Scaffolding project", || {
        Command::new("cargo")
            .arg("new")
            .arg("--bin")
            .arg(name)
            .status()
            .context("Failed to create cargo project")?;

        fs::write(path.join("agent.yaml"), DEFAULT_YAML.trim_start())?;
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
        tokio_table.insert("version", Value::from("1.45.1"));
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
                a.push("gem");
                a
            }),
        );
        deps.insert("autogpt", Item::Value(Value::InlineTable(autogpt_table)));

        let features = doc["features"].or_insert(Item::Table(Table::new()));
        let f_table = features.as_table_mut().unwrap();

        let mut gem_array = Array::default();
        gem_array.push("autogpt/gem");
        f_table.insert("gem", Item::Value(Value::Array(gem_array)));

        for feat in ["net", "mem", "oai", "cld", "xai"] {
            f_table.insert(feat, Item::Value(Value::Array(Array::default())));
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
    let yaml_content = std::fs::read_to_string(&yaml_path).context("Failed to read agent.yaml")?;

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
    let existing = std::fs::read_to_string(profile_path).unwrap_or_default();

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
