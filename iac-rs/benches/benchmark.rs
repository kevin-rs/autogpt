use criterion::{Criterion, criterion_group, criterion_main};
use iac_rs::prelude::*;
use plotters::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;
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
        let now = Instant::now();

        agent1.broadcast(black_box("test message")).await.unwrap();

        let elapsed = black_box(now.elapsed().as_micros() as u64);
        latencies.push(elapsed);
    }

    let mean = latencies.iter().copied().sum::<u64>() as f64 / latencies.len() as f64;

    let median = {
        let mut sorted = latencies.clone();
        sorted.sort_unstable();
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) as f64 / 2.0
        } else {
            sorted[mid] as f64
        }
    };

    let root_area = BitMapBackend::new("iac_benchmark.png", (1200, 600)).into_drawing_area();
    root_area.fill(&WHITE).unwrap();

    let max_latency = *latencies.iter().max().unwrap_or(&0);
    let min_latency = *latencies.iter().min().unwrap_or(&0);

    let mut chart = ChartBuilder::on(&root_area)
        .caption("Latency per Iteration (µs)", ("sans-serif", 30))
        .margin(30)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..latencies.len(), min_latency..max_latency)
        .unwrap();

    chart
        .configure_mesh()
        .x_desc("Iteration")
        .y_desc("Latency (µs)")
        .axis_desc_style(("sans-serif", 20))
        .light_line_style(&TRANSPARENT)
        .draw()
        .unwrap();

    chart
        .draw_series(LineSeries::new(
            latencies.iter().enumerate().map(|(i, &v)| (i, v)),
            &RED,
        ))
        .unwrap()
        .label("Latency")
        .legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], &RED));

    chart
        .draw_series(std::iter::once(PathElement::new(
            [(0, mean as u64), (latencies.len(), mean as u64)],
            ShapeStyle {
                color: BLUE.mix(0.6).to_rgba(),
                filled: false,
                stroke_width: 2,
            },
        )))
        .unwrap()
        .label("Mean")
        .legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], &BLUE));

    chart
        .draw_series(std::iter::once(PathElement::new(
            [(0, median as u64), (latencies.len(), median as u64)],
            ShapeStyle {
                color: GREEN.mix(0.6).to_rgba(),
                filled: false,
                stroke_width: 2,
            },
        )))
        .unwrap()
        .label("Median")
        .legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], &GREEN));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .label_font(("sans-serif", 18))
        .draw()
        .unwrap();
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
