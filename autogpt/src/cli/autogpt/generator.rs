use crate::cli::autogpt::ast::AgentConfig;
use anyhow::Result;
use convert_case::{Case, Casing};
use std::fs;
use std::path::Path;

pub fn generate_code(config: &AgentConfig, out: &Path) -> Result<()> {
    let struct_name = config.name.to_case(Case::UpperCamel);
    let code = format!(
        r#"#![allow(unused)]

use autogpt::prelude::*;
use std::borrow::Cow;

#[derive(Debug, Default, Auto)]
pub struct {name} {{
    objective: Cow<'static, str>,
    position: Cow<'static, str>,
    status: Status,
    agent: AgentGPT,
    client: ClientType,
    memory: Vec<Communication>,
}}

#[async_trait]
impl Executor for {name} {{
    async fn execute<'a>(
        &'a mut self,
        tasks: &'a mut Task,
        execute: bool,
        browse: bool,
        max_tries: u64,
    ) -> Result<()> {{
        let prompt = self.agent.objective().clone();
        let response = self.send_request(prompt.as_ref()).await?;

        self.agent.add_communication(Communication {{
            role: "{role}".into(),
            content: response.clone().into(),
        }});

        println!("{{}}", response);
        Ok(())
    }}
}}

#[tokio::main]
async fn main() {{
    let agent = {name}::new(
        "{prompt}".into(),
        "{position}".into()
    );

    let autogpt = AutoGPT::default()
        .with(agents![agent])
        .build()
        .expect("Build failed");

    match autogpt.run().await {{
        Ok(response) => println!("{{}}", response),
        Err(err) => eprintln!("Agent error: {{:?}}", err),
    }}
}}
"#,
        name = struct_name,
        role = config.role,
        prompt = config.prompt,
        position = config.position,
    );

    fs::write(out, code)?;
    Ok(())
}
