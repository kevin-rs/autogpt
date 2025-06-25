use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let objective = r#"Generate a diagram for a simple web application running on Kubernetes.
    It consists of a single Deployment with 2 replicas, a Service to expose the Deployment,
    and an Ingress to route external traffic. Also include a basic monitoring setup
    with Prometheus and Grafana."#;
    let position = "Lead UX/UI Designer";

    let agent = ArchitectGPT::new(objective, position).await;

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

    let guard = autogpt.agents[0].lock().await;
    let agent = guard.get_agent();

    println!("Memory length: {}", agent.memory().len());
    println!("First memory role: {}", agent.memory()[0].role);
    println!("Second memory role: {}", agent.memory()[1].role);

    println!("Agent status: {:?}", agent.status());
}
