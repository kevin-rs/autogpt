use autogpt::orchestrator::Orchestrator;
use iac_rs::prelude::*;
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::test]
#[ignore]
async fn test_orchestrator_create_and_terminate_agent() {
    // Skipping this test for now - it's failing in CircleCI but passing locally.
    // TODO: Look into why CircleCI doesn't allow task spawning.
    let filter = filter::LevelFilter::INFO;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let signer = Signer::new(KeyPair::generate());
    let verifier = Verifier::new(vec![]);

    let mut orchestrator = Orchestrator::new("orchestrator".to_string(), signer.clone(), verifier)
        .await
        .unwrap();
    let agents = orchestrator.agents.clone();

    tokio::spawn(async move {
        let _ = orchestrator.run().await;
    });

    let create_msg = Message {
        from: "tester".into(),
        to: "ArchitectGPT".into(),
        msg_type: MessageType::Create,
        ..Default::default()
    };
    let client = Client::connect("127.0.0.1:8443", signer.clone())
        .await
        .unwrap();

    client.send(create_msg).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let locked_agents = agents.lock().await;
    assert!(locked_agents.contains_key("ArchitectGPT"));

    drop(locked_agents);

    let terminate_msg = Message {
        from: "tester".into(),
        to: "ArchitectGPT".into(),
        msg_type: MessageType::Terminate,
        ..Default::default()
    };
    client.send(terminate_msg).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let locked_agents = agents.lock().await;
    assert!(!locked_agents.contains_key("ArchitectGPT"));
}
