use criterion::{Criterion, criterion_group, criterion_main};
use iac_rs::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio::time::Duration;

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

async fn setup_agents(addr: &str) -> (Arc<Agent>, Arc<Agent>) {
    let signer1 = Signer::new(KeyPair::generate());
    let signer2 = Signer::new(KeyPair::generate());

    let verifier = Verifier::new(vec![signer1.verifying_key(), signer2.verifying_key()]);

    tokio::spawn({
        async move {
            let mut server = Server::bind("0.0.0.0:4555").await.unwrap();
            server.run(verifier).await.unwrap();
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client1 = Client::connect(addr, signer1.clone()).await.unwrap();
    let client2 = Client::connect(addr, signer2.clone()).await.unwrap();

    let mut clients1 = HashMap::new();
    clients1.insert("agent-2".into(), Arc::new(Mutex::new(client1)));

    let mut clients2 = HashMap::new();
    clients2.insert("agent-1".into(), Arc::new(Mutex::new(client2)));

    let agent1 = Arc::new(Agent {
        id: "agent-1".into(),
        signer: signer1,
        verifiers: HashMap::new(),
        addr: addr.into(),
        clients: clients1,
        server: None,
        heartbeat_interval: Duration::from_millis(500),
        peer_addresses: HashMap::new(),
    });

    let agent2 = Arc::new(Agent {
        id: "agent-2".into(),
        signer: signer2,
        verifiers: HashMap::new(),
        addr: addr.into(),
        clients: clients2,
        server: None,
        heartbeat_interval: Duration::from_millis(500),
        peer_addresses: HashMap::new(),
    });

    (agent1, agent2)
}

async fn bench_iac_async(agent1: Arc<Agent>) {
    let mut latencies = Vec::with_capacity(1_000);

    for _ in 0..1_000 {
        let now = tokio::time::Instant::now();
        agent1.broadcast("test message").await.unwrap();
        latencies.push(now.elapsed().as_micros());
    }

    let csv = latencies
        .into_iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("iac_benchmark.csv");
    std::fs::write(&path, csv).unwrap();
}

fn bench_iac(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let agent1 = rt.block_on(async { setup_agents("127.0.0.1:4555").await.0 });

    c.bench_function("IAC broadcast + CSV log", |b| {
        b.to_async(&rt).iter(|| bench_iac_async(agent1.clone()))
    });
}

criterion_group!(benches, bench_iac);
criterion_main!(benches);
