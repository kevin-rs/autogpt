use autogpt::prelude::*;
use iac_rs::prelude::*;
use std::collections::HashMap;
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::test]
async fn test_agents_collaborations() -> Result<()> {
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

    let net1 = Arc::new(AgentNet {
        id: "designer".into(),
        signer: signer1.clone(),
        verifiers: HashMap::new(),
        addr: addr.into(),
        clients: clients1,
        server: None,
        heartbeat_interval: Duration::from_secs(1),
        peer_addresses: HashMap::new(),
    });

    let net2 = Arc::new(AgentNet {
        id: "frontend".into(),
        signer: signer2.clone(),
        verifiers: HashMap::new(),
        addr: addr.into(),
        clients: clients2,
        server: None,
        heartbeat_interval: Duration::from_secs(1),
        peer_addresses: HashMap::new(),
    });

    let designer = AgentGPT {
        id: "designer".into(),
        capabilities: vec![Capability::CodeGen].into_iter().collect(),
        network: Some(net1.clone()),
        ..Default::default()
    };

    let frontend = AgentGPT {
        id: "frontend".into(),
        capabilities: vec![Capability::UIDesign].into_iter().collect(),
        network: Some(net2.clone()),
        ..Default::default()
    };

    designer
        .swarm
        .lock()
        .await
        .register_local(
            Collaborator::Local(Arc::new(Mutex::new(designer.clone()))),
            designer.capabilities.iter().cloned().collect(),
        )
        .await;

    frontend
        .swarm
        .lock()
        .await
        .register_local(
            Collaborator::Local(Arc::new(Mutex::new(frontend.clone()))),
            frontend.capabilities.iter().cloned().collect(),
        )
        .await;

    let d_swarm = designer.swarm.lock().await;
    assert!(d_swarm.locals.contains_key("designer"));
    assert!(
        d_swarm
            .cap_index
            .get(&Capability::CodeGen)
            .unwrap()
            .contains(&"designer".to_string()),
    );
    drop(d_swarm);

    let f_swarm = frontend.swarm.lock().await;
    assert!(f_swarm.locals.contains_key("frontend"));
    assert!(
        f_swarm
            .cap_index
            .get(&Capability::UIDesign)
            .unwrap()
            .contains(&"frontend".to_string()),
    );
    drop(f_swarm);

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

    let d_swarm = designer.swarm.lock().await;
    assert!(d_swarm.remotes.contains_key("frontend"));
    assert!(
        d_swarm
            .cap_index
            .get(&Capability::UIDesign)
            .unwrap()
            .contains(&"frontend".to_string())
    );
    drop(d_swarm);

    let f_swarm = frontend.swarm.lock().await;
    assert!(f_swarm.remotes.contains_key("designer"));
    assert!(
        f_swarm
            .cap_index
            .get(&Capability::CodeGen)
            .unwrap()
            .contains(&"designer".to_string())
    );
    drop(f_swarm);

    let task = Task {
        description: "Design a UI component".into(),
        ..Default::default()
    };

    let result = frontend
        .swarm
        .lock()
        .await
        .assign_task_lb(&Capability::CodeGen, task.clone())
        .await;

    assert!(result.is_ok(), "Task assignment failed");

    Ok(())
}
