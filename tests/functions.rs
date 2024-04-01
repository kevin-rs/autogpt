use anyhow::Result;
use autogpt::agents::agent::AgentGPT;
use autogpt::common::utils::{Route, Scope, Tasks};
use autogpt::traits::functions::Functions;
use serde_json::json;
use std::borrow::Cow;
use tracing::info;
use tracing_subscriber::{filter, fmt, prelude::*, reload};

pub struct MockFunctions {
    agent: AgentGPT,
}

impl Functions for MockFunctions {
    fn get_agent(&self) -> &AgentGPT {
        &self.agent
    }

    async fn execute(&mut self, tasks: &mut Tasks, _execute: bool, max_tries: u64) -> Result<()> {
        info!("Executing tasks: {:?}", tasks.clone());

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        tasks.description = "Updated description".into();

        Ok(())
    }
}

#[tokio::test]
async fn test_functions_execution() {
    let filter = filter::LevelFilter::INFO;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let objective = "Objective";
    let position = "Position";
    let agent = AgentGPT::new_borrowed(objective, position);

    let mut tasks = Tasks {
        description: Cow::Borrowed("Tasks"),
        scope: Some(Scope {
            crud: true,
            auth: false,
            external: true,
        }),
        urls: Some(vec![Cow::Borrowed("https://kevin-rs.dev")]),
        backend_code: Some(Cow::Borrowed("fn main() {}")),
        frontend_code: None,
        api_schema: Some(vec![
            Route {
                dynamic: Cow::Borrowed("no"),
                method: Cow::Borrowed("GET"),
                body: json!({}),
                response: json!({}),
                path: Cow::Borrowed("/path"),
            },
            Route {
                dynamic: Cow::Borrowed("yes"),
                method: Cow::Borrowed("POST"),
                body: json!({"key": "value"}),
                response: json!({"success": true}),
                path: Cow::Borrowed("/path"),
            },
        ]),
    };

    let mut functions = MockFunctions { agent };

    let result = functions.execute(&mut tasks, true, 3).await;

    assert!(result.is_ok());
}
