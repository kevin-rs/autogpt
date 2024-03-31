use anyhow::Result;
use autogpt::agents::agent::AgentGPT;
use autogpt::common::utils::{Route, Scope, Tasks};
use autogpt::traits::functions::Functions;
use serde_json::json;
use std::borrow::Cow;
use tracing::debug;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub struct MockFunctions {
    agent: AgentGPT,
}

impl Functions for MockFunctions {
    fn get_agent(&self) -> &AgentGPT {
        &self.agent
    }

    async fn execute(&mut self, tasks: &mut Tasks) -> Result<()> {
        debug!("Executing tasks: {:?}", tasks.clone());

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        tasks.description = "Updated description".into();

        Ok(())
    }
}

#[tokio::test]
async fn test_functions_execution() {
    tracing_subscriber::registry().with(fmt::layer()).init();
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

    let result = functions.execute(&mut tasks).await;

    assert!(result.is_ok());
}
