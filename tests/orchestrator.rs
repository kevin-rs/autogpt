use autogpt::message::Message;
use autogpt::orchestrator::Orchestrator;
use tokio::sync::mpsc;
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::test]
async fn test_orchestrator_create_and_terminate_agent() {
    let filter = filter::LevelFilter::INFO;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let (tx, rx) = mpsc::channel(8);

    let orchestrator = Orchestrator::new(rx).await.unwrap();
    let agents = orchestrator.agents.clone();

    tokio::spawn(async move {
        let _ = orchestrator.run().await;
    });

    let create_msg = Message {
        from: "tester".into(),
        to: "ArchitectGPT".into(),
        msg_type: "create".into(),
        payload_json: "".into(),
        auth_token: "securetoken".into(),
    };
    tx.send(create_msg).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let locked_agents = agents.lock().await;
    assert!(locked_agents.contains_key("ArchitectGPT"));

    drop(locked_agents);

    let terminate_msg = Message {
        from: "tester".into(),
        to: "ArchitectGPT".into(),
        msg_type: "terminate".into(),
        payload_json: "".into(),
        auth_token: "securetoken".into(),
    };
    tx.send(terminate_msg).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let locked_agents = agents.lock().await;
    assert!(!locked_agents.contains_key("ArchitectGPT"));
}
