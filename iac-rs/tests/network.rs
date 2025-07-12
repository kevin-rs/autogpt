use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use anyhow::Result;
use iac_rs::prelude::*;
use tokio::sync::Mutex;
use tokio::time;

#[derive(Default, Debug)]
struct MessageLog {
    pub messages: Vec<String>,
}

#[derive(AutoNet)]
pub struct Agent {
    pub id: Cow<'static, str>,
    pub signer: Signer,
    pub verifiers: HashMap<String, Verifier>,
    pub addr: String,
    pub clients: HashMap<String, Arc<Mutex<Client>>>,
    pub server: Option<Arc<Mutex<Server>>>,
    pub heartbeat_interval: Duration,
    pub peer_addresses: HashMap<String, String>,
}

#[tokio::test]
async fn test_heartbeat() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let addr = "0.0.0.0:4555";
    let client_addr = "127.0.0.1:4555";

    let signer1 = Signer::new(KeyPair::generate());
    let signer2 = Signer::new(KeyPair::generate());
    let signer1_pk = signer1.verifying_key();
    let signer2_pk = signer2.verifying_key();

    let verifier = Verifier::new(vec![signer1_pk, signer2_pk]);

    let message_log = Arc::new(StdMutex::new(MessageLog::default()));
    let log_clone = Arc::clone(&message_log);

    tokio::spawn(async move {
        let mut server = Server::bind(addr).await.unwrap();
        server.set_handler(move |(msg, _conn)| {
            let log_clone = Arc::clone(&log_clone);
            async move {
                let mut log = log_clone.lock().unwrap();
                log.messages
                    .push(format!("[{} -> {}] {:?}", msg.from, msg.to, msg.msg_type));
                Ok(())
            }
        });
        server.run(verifier).await.unwrap();
    });

    time::sleep(Duration::from_millis(200)).await;

    let client1 = Client::connect(client_addr, signer1.clone()).await?;

    let mut clients1 = HashMap::new();
    clients1.insert("agent-2".to_string(), Arc::new(Mutex::new(client1)));

    let mut peer_addresses1 = HashMap::new();
    peer_addresses1.insert("agent-2".to_string(), client_addr.to_string());

    let mut verifiers1 = HashMap::new();
    verifiers1.insert("agent-2".to_string(), Verifier::new(vec![signer1_pk]));

    let agent_1 = Agent {
        id: "agent-1".into(),
        signer: signer1.clone(),
        verifiers: verifiers1,
        addr: client_addr.to_string(),
        clients: clients1,
        server: None,
        heartbeat_interval: Duration::from_millis(300),
        peer_addresses: peer_addresses1,
    };

    let client2 = Client::connect(client_addr, signer2.clone()).await?;

    let mut clients2 = HashMap::new();
    clients2.insert("agent-1".to_string(), Arc::new(Mutex::new(client2)));

    let mut peer_addresses2 = HashMap::new();
    peer_addresses2.insert("agent-1".to_string(), client_addr.to_string());

    let mut verifiers2 = HashMap::new();
    verifiers2.insert("agent-1".to_string(), Verifier::new(vec![signer2_pk]));

    let agent_2 = Agent {
        id: "agent-2".into(),
        signer: signer2,
        verifiers: verifiers2,
        addr: client_addr.to_string(),
        clients: clients2,
        server: None,
        heartbeat_interval: Duration::from_millis(300),
        peer_addresses: peer_addresses2,
    };

    let agent_1 = Arc::new(agent_1);
    let agent_1_clone = Arc::clone(&agent_1);

    let hb1 = tokio::spawn(async move {
        agent_1_clone.heartbeat().await;
    });

    let agent_2 = Arc::new(agent_2);
    let agent_2_clone = Arc::clone(&agent_2);

    let hb2 = tokio::spawn(async move {
        agent_2_clone.heartbeat().await;
    });

    time::sleep(Duration::from_secs(2)).await;

    hb1.abort();
    hb2.abort();

    let log = message_log.lock().unwrap();

    assert!(
        log.messages
            .iter()
            .any(|msg| msg.contains("[agent-1 -> agent-2] Ping")
                || msg.contains("[agent-2 -> agent-1] Ping"))
    );

    Ok(())
}

#[tokio::test]
async fn test_broadcast() -> Result<()> {
    let addr = "0.0.0.0:4556";
    let client_addr = "127.0.0.1:4556";

    let signer1 = Signer::new(KeyPair::generate());
    let signer2 = Signer::new(KeyPair::generate());
    let signer1_pk = signer1.verifying_key();
    let signer2_pk = signer2.verifying_key();

    let verifier = Verifier::new(vec![signer1_pk, signer2_pk]);

    let message_log = Arc::new(StdMutex::new(MessageLog::default()));
    let log_clone = Arc::clone(&message_log);

    tokio::spawn(async move {
        let mut server = Server::bind(addr).await.unwrap();
        server.set_handler(move |(msg, _conn)| {
            let log_clone = Arc::clone(&log_clone);
            async move {
                let mut log = log_clone.lock().unwrap();
                log.messages
                    .push(format!("[{} -> {}] {:?}", msg.from, msg.to, msg.msg_type));
                Ok(())
            }
        });
        server.run(verifier).await.unwrap();
    });

    time::sleep(Duration::from_millis(200)).await;

    let client1 = Client::connect(client_addr, signer1.clone()).await?;

    let mut clients1 = HashMap::new();
    clients1.insert("agent-2".to_string(), Arc::new(Mutex::new(client1)));

    let mut peer_addresses1 = HashMap::new();
    peer_addresses1.insert("agent-2".to_string(), client_addr.to_string());

    let mut verifiers1 = HashMap::new();
    verifiers1.insert("agent-2".to_string(), Verifier::new(vec![signer1_pk]));

    let agent_1 = Agent {
        id: "agent-1".into(),
        signer: signer1.clone(),
        verifiers: verifiers1,
        addr: client_addr.to_string(),
        clients: clients1,
        server: None,
        heartbeat_interval: Duration::from_millis(300),
        peer_addresses: peer_addresses1,
    };

    let client2 = Client::connect(client_addr, signer2.clone()).await?;

    let mut clients2 = HashMap::new();
    clients2.insert("agent-1".to_string(), Arc::new(Mutex::new(client2)));

    let mut peer_addresses2 = HashMap::new();
    peer_addresses2.insert("agent-1".to_string(), client_addr.to_string());

    let mut verifiers2 = HashMap::new();
    verifiers2.insert("agent-1".to_string(), Verifier::new(vec![signer2_pk]));

    let agent_1 = Arc::new(agent_1);

    let agent_1_clone = Arc::clone(&agent_1);
    let broadcast: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async move {
        agent_1_clone.broadcast("greetings from agent 1").await?;
        Ok(())
    });

    time::sleep(Duration::from_secs(1)).await;
    broadcast.abort();

    time::sleep(Duration::from_secs(1)).await;
    broadcast.abort();

    let log = message_log.lock().unwrap();

    assert!(
        log.messages
            .iter()
            .any(|msg| msg.contains("[agent-1 -> agent-2] Broadcast"))
    );

    Ok(())
}
