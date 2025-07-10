use autogpt::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::test]
async fn test_agents_collaboration() -> Result<()> {
    let filter = filter::LevelFilter::DEBUG;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let addr = "127.0.0.1:4555";
    let signer1 = Signer::new(KeyPair::generate());
    let signer2 = Signer::new(KeyPair::generate());

    let verifier = Verifier::new(vec![signer1.verifying_key(), signer2.verifying_key()]);

    tokio::spawn(async move {
        let mut server = Server::bind("0.0.0.0:4555").await.unwrap();
        server.run(verifier).await.unwrap();
    });

    let client1 = Client::connect(addr, signer1.clone()).await?;
    let client2 = Client::connect(addr, signer2.clone()).await?;

    let mut clients1 = HashMap::new();
    clients1.insert("frontend".into(), Arc::new(Mutex::new(client1)));
    let mut clients2 = HashMap::new();
    clients2.insert("designer".into(), Arc::new(Mutex::new(client2)));

    let mut designer = AgentGPT::new("designer".into(), "objective".into());
    let mut frontend = AgentGPT::new("frontend".into(), "objective".into());

    designer.signer = signer1.clone();
    designer.clients = clients1.clone();
    designer.addr = addr.into();

    frontend.signer = signer2.clone();
    frontend.clients = clients2.clone();
    frontend.addr = addr.into();

    designer.capabilities.insert(Capability::CodeGen);
    frontend.capabilities.insert(Capability::UIDesign);

    designer
        .register_local(
            Collaborator::Local(Arc::new(Mutex::new(designer.clone()))),
            designer.capabilities.iter().cloned().collect(),
        )
        .await;
    frontend
        .register_local(
            Collaborator::Local(Arc::new(Mutex::new(frontend.clone()))),
            frontend.capabilities.iter().cloned().collect(),
        )
        .await;

    assert!(
        designer
            .local_collaborators
            .contains_key(designer.id.as_ref())
    );
    assert!(
        designer
            .cap_index
            .get(&Capability::CodeGen)
            .unwrap()
            .contains(&designer.id.to_string())
    );

    assert!(
        frontend
            .local_collaborators
            .contains_key(frontend.id.as_ref())
    );
    assert!(
        frontend
            .cap_index
            .get(&Capability::UIDesign)
            .unwrap()
            .contains(&frontend.id.to_string())
    );

    designer.broadcast_capabilities().await?;
    frontend.broadcast_capabilities().await?;

    designer
        .receive_message(AgentMessage::CapabilityAdvert {
            sender_id: "frontend".into(),
            capabilities: frontend.capabilities.iter().cloned().collect(),
        })
        .await?;
    frontend
        .receive_message(AgentMessage::CapabilityAdvert {
            sender_id: "designer".into(),
            capabilities: designer.capabilities.iter().cloned().collect(),
        })
        .await?;

    assert!(designer.remote_collaborators.contains_key("frontend"));
    assert!(
        designer
            .cap_index
            .get(&Capability::UIDesign)
            .unwrap()
            .contains(&"frontend".to_string())
    );
    assert!(frontend.remote_collaborators.contains_key("designer"));
    assert!(
        frontend
            .cap_index
            .get(&Capability::CodeGen)
            .unwrap()
            .contains(&"designer".to_string())
    );

    let task = Task {
        description: "Design a UI component".into(),
        ..Default::default()
    };
    let result = frontend
        .assign_task_lb(&Capability::CodeGen, task.clone())
        .await;
    assert!(result.is_ok(), "Task assignment failed");

    Ok(())
}
