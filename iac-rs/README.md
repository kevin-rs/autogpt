<div align="center">

# 🛰️ IAC Protocol

[![Protocol Spec](https://img.shields.io/badge/Spec-IAC-purple.svg)](https://github.com/kevin-rs/autogpt)
[![Language](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/Status-Experimental-blueviolet.svg)](https://github.com/kevin-rs/autogpt)
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-brightgreen.svg)](https://github.com/wiseaidev)

![logo](https://raw.githubusercontent.com/kevin-rs/autogpt/refs/heads/main/iac-rs/assets/logo.webp)

</div>

## 📜 Intro

**IAC** (Inter & Intra Agent Communication) is a next-generation agent coordination protocol designed for large-scale, decentralized, and asynchronous environments. It provides a structured, language-agnostic command interface to delegate tasks across orchestrated AI agents and more. Built from first principles for the modern AI ecosystem, IAC decouples agent control from execution logic, enabling clean separation of roles, flexible topologies, and modular deployment patterns, from cloud AI clusters to local edge devices.

## ⚡ Why IAC?

- **Intent-Based Semantics**: Commands are passed in a human-readable, strongly typed format enabling clear delegation of responsibilities.
- **Distributed Native**: Designed for horizontal scalability and federated agent networks with low-latency routing.
- **Fast Bootstrapping**: Instantiate and link complex agents and services on demand using expressive slash commands.
- **Minimal Overhead**: IAC avoids bulky RPC-style overhead in favor of event-driven, stateless messaging pipelines.
- **Transparent Execution**: Each interaction step is verifiable, traceable, and semantically rich, suitable for audit and introspection.

## 🧠 Philosophy

IAC doesn't simply optimize communication, it **redefines** how intent flows between autonomous agents. It emphasizes **explicit structure**, **declarative commands**, and **modular agent roles**, moving away from the fragile, opaque APIs of legacy systems. This protocol enables new forms of **agent collaboration**, **autonomous orchestration**, and **meta-control**, promoting architectures that think, plan, and act.

## 📦 Usage

To get started with IAC in your project, add the crate to start a server and connect a client that sends a signed message:

```rust
use iac_rs::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "127.0.0.1:4433";
    let server_addr = "0.0.0.0:4433";

    tokio::spawn(async move {
        let mut server = Server::bind(server_addr).await?;
        let verifier = Verifier::new(vec![KeyPair::generate().pk]);
        server.run(verifier).await?;
        Ok::<(), anyhow::Error>(())
    });

    let signer = Signer::new(KeyPair::generate());
    let client = Client::connect(addr, signer.clone()).await?;

    let mut msg = Message {
        msg_id: 1,
        from: "client".to_string(),
        to: "server".to_string(),
        signature: vec![],
        ..Default::default()
    };

    msg.sign(&signer)?;
    client.send(msg).await?;

    Ok(())
}
```

This demonstrates:

- Setting up a local server with a public key verifier
- Connecting a client with a generated signer key
- Creating and signing a message
- Sending the message over the IAC protocol

### 🌐 Agentic Network

```rust
use iac_rs::prelude::*;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::borrow::Cow;

// Use the `AutoNet` macro to automatically implement the `Network` trait and enable IAC protocol support.
// The struct must define at least these fields for `AutoNet` to function correctly:
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "0.0.0.0:4555";
    let client_addr = "127.0.0.1:4555";

    // Generate keypairs for agents
    let signer1 = Signer::new(KeyPair::generate());
    let signer2 = Signer::new(KeyPair::generate());

    // Server verifier (accepts both agents' keys)
    let verifier = Verifier::new(vec![signer1.verifying_key(), signer2.verifying_key()]);

    // Start server
    tokio::spawn(async move {
        let mut server = Server::bind(addr).await.unwrap();
        server.run(verifier).await.unwrap();
    });

    // Agent 1 setup
    let client1 = Client::connect(client_addr, signer1.clone()).await?;
    let mut clients1 = HashMap::new();
    clients1.insert("agent-2".into(), Arc::new(Mutex::new(client1)));

    let agent1 = Arc::new(Agent {
        id: "agent-1".into(),
        signer: signer1,
        verifiers: HashMap::new(),
        addr: client_addr.into(),
        clients: clients1,
        server: None,
        heartbeat_interval: Duration::from_millis(500),
        peer_addresses: HashMap::new(),
    });

    // Agent 2 setup
    let client2 = Client::connect(client_addr, signer2.clone()).await?;
    let mut clients2 = HashMap::new();
    clients2.insert("agent-1".into(), Arc::new(Mutex::new(client2)));

    let agent2 = Arc::new(Agent {
        id: "agent-2".into(),
        signer: signer2,
        verifiers: HashMap::new(),
        addr: client_addr.into(),
        clients: clients2,
        server: None,
        heartbeat_interval: Duration::from_millis(500),
        peer_addresses: HashMap::new(),
    });

    // Start heartbeat tasks
    let hb1 = {
        let agent = Arc::clone(&agent1);
        tokio::spawn(async move {
            agent.heartbeat().await;
        })
    };

    let hb2 = {
        let agent = Arc::clone(&agent2);
        tokio::spawn(async move {
            agent.heartbeat().await;
        })
    };

    // Agent 1 broadcasts a message
    agent1.broadcast("hello from agent-1").await?;

    hb1.abort();
    hb2.abort();

    Ok(())
}
```

This shows:

- How agents send **heartbeat** messages.
- How agents send **broadcast** messages.

## 🚀 Performance

![benchmark](https://raw.githubusercontent.com/kevin-rs/autogpt/refs/heads/main/iac-rs/benches/iac_benchmark.png)

Through benchmarks (by running `cargo bench`), we found that **IAC achieves exceptional speed and efficiency**, with **sub-millisecond signed broadcasts** and tight tail latencies. Median message latency sits around **296 µs**, with the **mean at approximately 312 µs**, and the **99th percentile remaining under 650 µs** across 1,000 async roundtrips. Built on QUIC, Ed25519, and Protobuf, IAC eliminates traditional overhead via **zero-RTT transport**, **concurrent streams**, and **fully async execution**. This makes IAC ideal for **high-frequency agent coordination**, **distributed task orchestration**, and **real-time multi-agent systems**, all with cryptographic guarantees and minimal delay.

## 🤝 Contributing

We welcome all contributions: ideas, issues, improvements, documentation, or code!

1. Fork the repository.
1. Create a new feature branch.
1. Submit a pull request.

Let's build the next-gen agent protocol together.

## 📜 License

IAC is licensed under the [MIT License](./LICENSE.md). You are free to use, modify, and distribute the protocol in your own applications.
