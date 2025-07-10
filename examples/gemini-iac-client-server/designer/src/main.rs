use autogpt::prelude::*;
use std::collections::HashMap;
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, reload};

#[tokio::main]
async fn main() -> Result<()> {
    let filter = LevelFilter::DEBUG;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let addr = "127.0.0.1:4555";
    let signer = Signer::new(KeyPair::generate());
    let client = Client::connect(addr, signer.clone()).await?;

    let mut clients = HashMap::new();
    clients.insert("frontend".into(), Arc::new(Mutex::new(client.clone())));

    let mut agent = AgentGPT::new("Design software".into(), "designer".into());
    agent.signer = signer;
    agent.clients = clients;
    agent.addr = addr.into();
    agent.capabilities.insert(Capability::CodeGen);

    let public_key_bytes = agent.signer.verifying_key().as_slice().to_vec();

    let register_key_msg = IacMessage {
        from: agent.id.to_string(),
        to: "server".into(),
        msg_type: MessageType::RegisterKey.into(),
        extra_data: public_key_bytes,
        ..Default::default()
    };

    client.send(register_key_msg).await?;

    agent
        .register_local(
            Collaborator::Local(Arc::new(Mutex::new(agent.clone()))),
            agent.capabilities.iter().cloned().collect(),
        )
        .await;

    agent.broadcast_capabilities().await?;

    agent
        .receive_message(AgentMessage::CapabilityAdvert {
            sender_id: "frontend".into(),
            capabilities: agent.capabilities.iter().cloned().collect(),
        })
        .await?;

    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        agent.heartbeat().await;
    }
}
