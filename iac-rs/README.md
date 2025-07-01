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
async fn main() -> Result<()> {
    let addr = "127.0.0.1:4433";
    let server_addr = "0.0.0.0:4433";

    tokio::spawn(async move {
        let mut server = Server::bind(server_addr).await?;
        let verifier = Verifier::new(KeyPair::generate().pk);
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

## 🤝 Contributing

We welcome all contributions: ideas, issues, improvements, documentation, or code!

1. Fork the repository.
1. Create a new feature branch.
1. Submit a pull request.

Let's build the next-gen agent protocol together.

## 📜 License

IAC is licensed under the [MIT License](./LICENSE.md). You are free to use, modify, and distribute the protocol in your own applications.
