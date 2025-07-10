#![allow(unused)]

use autogpt::prelude::*;

/// To be compatible with AutoGPT, an agent must implement the `Agent`,
/// `Functions`, and `AsyncFunctions` traits.
/// These traits can be automatically derived using the `Auto` macro.
/// The agent struct must contain at least the following fields.
#[derive(Debug, Default, Auto)]
pub struct CustomAgent {
    objective: Cow<'static, str>,
    position: Cow<'static, str>,
    status: Status,
    agent: AgentGPT,
    client: ClientType,
    memory: Vec<Communication>,
}

#[async_trait]
impl Executor for CustomAgent {
    async fn execute<'a>(
        &'a mut self,
        tasks: &'a mut Task,
        execute: bool,
        browse: bool,
        max_tries: u64,
    ) -> Result<()> {
        let prompt = self.agent.objective().clone();
        let response = self.send_request(prompt.as_ref()).await?;

        self.agent.add_communication(Communication {
            role: "assistant".into(),
            content: response.clone().into(),
        });

        println!("{}", response);

        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let prompt = r#"Generate a mermaid diagram for a simple web application running on Kubernetes.
    It consists of a single Deployment with 2 replicas, a Service to expose the Deployment,
    and an Ingress to route external traffic. Also include a basic monitoring setup
    with Prometheus and Grafana."#;
    let position = "Lead UX/UI Designer";

    let agent = CustomAgent::new(prompt.into(), position.into());

    let autogpt = AutoGPT::default()
        .with(agents![agent])
        .build()
        .expect("Failed to build AutoGPT");

    match autogpt.run().await {
        Ok(response) => {
            println!("{}", response);
        }
        Err(err) => {
            eprintln!("Agent error: {:?}", err);
        }
    }
}
