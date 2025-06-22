use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let mut autogpt = AutoGPTBuilder::default()
        .tools(vec![Tool::Diagram])
        .build()
        .expect("Failed to build AutoGPT");

    let msg = Message::from_text(
        "Generate a diagram for a simple web application running on Kubernetes.  It consists of a single Deployment with 2 replicas, a Service to expose the Deployment, and an Ingress to route external traffic.",
    );

    match autogpt.run(vec![msg]).await {
        Ok(response) => {
            println!("{}", response);
        }
        Err(err) => {
            eprintln!("Agent error: {:?}", err);
        }
    }
}
