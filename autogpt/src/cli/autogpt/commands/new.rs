use crate::cli::autogpt::utils::*;
use anyhow::{Context, Result, bail};
use std::{fs, path::Path, process::Command};
use toml_edit::{Array, DocumentMut, InlineTable, Item, Table, Value};

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
            format!("# {name}\n\nCreated with `agentc new`."),
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
        autogpt_table.insert("version", Value::from("0.1.9"));
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

    success(&format!("✅ Project created at ./{name}"));
    Ok(())
}

const DEFAULT_YAML: &str = r#"
name: agent_name
ai_provider: gemini
model: gemini-2.0-flash
position: Backend Engineer
role: user
prompt: |
  Describe a scalable microservice architecture.
"#;
