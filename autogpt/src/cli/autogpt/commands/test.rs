use crate::cli::autogpt::parser::parse_yaml;
use crate::cli::autogpt::utils::*;
use anyhow::Result;

pub fn handle_test() -> Result<()> {
    spinner("Validating YAML file", || {
        let config = parse_yaml("agent.yaml")?;
        println!("🔍 Agent(s) parsed:\n{config:#?}");
        Ok(())
    })?;

    success("✅ YAML is valid");
    Ok(())
}
