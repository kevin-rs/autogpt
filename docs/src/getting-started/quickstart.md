# Quickstart

This guide takes you from zero to a running AutoGPT agent in under 5 minutes, using the Gemini provider and the `ArchitectGPT` built-in agent.

## Prerequisites

- Rust 1.89+ (`rustup update stable`).
- A [Gemini API key](https://aistudio.google.com/app/api-keys).

## Step 1: Set your API key

```sh
export GEMINI_API_KEY=<your_gemini_api_key>
```

## Step 2: Create a new Rust project

```sh
cargo new my-agent && cd my-agent
```

Add AutoGPT to `Cargo.toml`:

```toml
[dependencies]
autogpt = { version = "0.4.4", features = ["gem", "gpt"] }
tokio   = { version = "1",   features = ["full"] }
```

## Step 3: Write your first agent

Replace `src/main.rs` with:

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let persona = "Lead UX/UI Designer";
    let behavior = r#"Generate a diagram for a simple web application running on Kubernetes.
    It consists of a single Deployment with 2 replicas, a Service to expose the Deployment,
    and an Ingress to route external traffic. Also include a basic monitoring setup
    with Prometheus and Grafana."#;

    let agent = ArchitectGPT::new(persona, behavior).await;

    let autogpt = AutoGPT::default()
        .with(agents![agent])
        .build()
        .expect("Failed to build AutoGPT");

    match autogpt.run().await {
        Ok(response) => println!("{}", response),
        Err(err)     => eprintln!("Agent error: {:?}", err),
    }
}
```

## Step 4: Run it

```sh
cargo run
```

AutoGPT calls Gemini, generates Python `diagrams`-library code, and writes it to:

```
workspace/architect/diagram.py
```

## Step 5: Render the diagram

```sh
./workspace/architect/.venv/bin/python ./workspace/architect/diagram.py
```

A PNG architecture diagram appears in `workspace/architect/`.

<div class="callout callout-tip">
<strong>💡 Tip</strong>
To switch to OpenAI instead of Gemini, set <code>export AI_PROVIDER=openai</code> and <code>export OPENAI_API_KEY=...</code>, then change the feature flag to <code>features = ["oai", "gpt"]</code>.
</div>

## Running Multiple Agents in Parallel

The `agents!` macro accepts any number of agents. AutoGPT runs them concurrently using Tokio tasks:

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let architect = ArchitectGPT::new(
        "Software Architect",
        "Design a microservices architecture for an e-commerce platform.",
    ).await;

    let backend = BackendGPT::new(
        "Backend Developer",
        "Generate a REST API for user authentication in Rust using axum.",
        "rust",
    ).await;

    let autogpt = AutoGPT::default()
        .with(agents![architect, backend])
        .build()
        .expect("Failed to build AutoGPT");

    match autogpt.run().await {
        Ok(msg) => println!("{}", msg),
        Err(e)  => eprintln!("{:?}", e),
    }
}
```

Both agents execute concurrently and their outputs land in their respective workspace directories.

## Next Steps

- [Configuration →](./configuration.md): set up memory, email, and image generation.
- [Custom Agents →](../sdk/custom-agents.md): build agents tailored to your domain.
- [Interactive Mode →](../modes/interactive.md): chat with AutoGPT from the terminal.
