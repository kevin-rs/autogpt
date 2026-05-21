# 📦 Installation

Welcome! AutoGPT offers seamless integration with both Cargo and Docker for easy installation and usage.

## 📦 Install From Registry

### ⚓ Using Cargo

To install AutoGPT CLI via Cargo, execute the following command:

```sh
cargo install autogpt --all-features
```

To install with specific features (e.g., MoP, Gemini, and MCP):

```sh
cargo install autogpt --features "cli,gem,mop,mcp"
```

To install with Agent Metacognition enabled:

```sh
cargo install autogpt --features "cli,gem,mta"
```

The `mta` (Meta-Thinking Agent) feature adds a self-evaluation phase to `GenericAgent`. After every task reflection, the agent records the outcome and, every 3 tasks or after 2+ consecutive failures, queries the LLM with a structured metacognition prompt to derive strategy adjustments for remaining tasks. The TUI status bar shows **MetaCognizing** during this phase.

### 🐳 Using Docker

To install and run the AutoGPT CLI via Docker, use the following command:

```sh
docker run -it \
  -e GEMINI_API_KEY=<your_gemini_api_key> \
  -e PINECONE_API_KEY=<Your_Pinecone_API_Key> \
  -e PINECONE_INDEX_URL=<Your_Pinecone_Index_URL> \
  --rm --name autogpt kevinrsdev/autogpt man
```

To install and run the OrchGPT CLI via Docker, use the following command:

```sh
docker run -it \
  -e GEMINI_API_KEY=<your_gemini_api_key> \
  -e PINECONE_API_KEY=<Your_Pinecone_API_Key> \
  -e PINECONE_INDEX_URL=<Your_Pinecone_Index_URL> \
  --rm --name orchgpt kevinrsdev/orchgpt
```

## 📦 Build From Source

Fork/Clone The Repo:

```sh
git clone https://github.com/wiseaidotdev/autogpt.git
```

Navigate to the core autogpt directory:

```sh
cd autogpt/autogpt
```

### ⚓ Using Cargo

To run OrchGPT CLI via Cargo, execute:

```sh
cargo run --all-features --bin orchgpt
```

To run AutoGPT CLI via Cargo, execute:

```sh
cargo run --all-features --bin autogpt
```

To run with Mixture of Providers:

```sh
cargo run --features "cli,gem,oai,mop" --bin autogpt -- --mixture
```

### 🐳 Using Docker

Install the [docker buildx plugin](https://docs.docker.com/build/concepts/overview/):

```sh
sudo apt-get update
sudo apt-get install docker-buildx-plugin
```

Once installed, build the `orchgpt` Docker container using BuildKit:

```sh
docker buildx build -f Dockerfile.orchgpt -t orchgpt .
```

Build the `autogpt` Docker container:

```sh
docker buildx build -f Dockerfile.autogpt -t autogpt .
```

Run the `orchgpt` container:

```sh
docker run -i \
  -e GEMINI_API_KEY=<your_gemini_api_key> \
  -e PINECONE_API_KEY=<Your_Pinecone_API_Key> \
  -e PINECONE_INDEX_URL=<Your_Pinecone_Index_URL> \
  -t orchgpt:latest
```

Run the `autogpt` container:

```sh
docker run -i \
  -e GEMINI_API_KEY=<your_gemini_api_key> \
  -e PINECONE_API_KEY=<Your_Pinecone_API_Key> \
  -e PINECONE_INDEX_URL=<Your_Pinecone_Index_URL> \
  -t autogpt:latest
```

Now, you can attach to the container:

```sh
$ docker ps
CONTAINER ID   IMAGE            COMMAND                  CREATED         STATUS         PORTS     NAMES
95bf85357513   autogpt:latest   "/usr/local/bin/auto…"   9 seconds ago   Up 8 seconds             autogpt

$ docker exec -it 95bf85357513 /bin/sh
~ $ ls
workspace
~ $ tree
.
└── workspace
    ├── architect
    │   └── diagram.py
    ├── backend
    │   ├── main.py
    │   └── template.py
    ├── designer
    └── frontend
        ├── main.py
        └── template.py
```

to stop the current container, open up a new terminal and run:

```sh
$ docker stop $(docker ps -q)
```

### 🚢 Using Compose V2

This project uses [**Docker Compose V2**](https://github.com/docker/compose) to define and manage two services:

- `autogpt` - an AutoGPT instance
- `orchgpt` - an orchestrator that interacts with the AutoGPT container

These services are built from separate custom Dockerfiles and run in isolated containers. Docker Compose sets up networking automatically, enabling communication between `autogpt` and `orchgpt` as if they were on the same local network.

#### 🚀 Build and Run

To build and start both services:

```sh
docker compose up --build
```

This will:

- Build both `autogpt` and `orchgpt` images using their respective Dockerfiles.
- Create and start the containers.
- Allow `autogpt` to communicate with `orchgpt`.

---

## 🧰 SDK Usage

The SDK offers a simple and flexible API for building and running intelligent agents in your applications. Before getting started, **make sure to configure the required environment variables**. For detailed setup, refer to the [Environment Variables Setup](#environment-variables-setup) section.

Once the environment is ready, you can quickly spin up an agent like so:

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let persona = "Lead UX/UI Designer";
    let behavior = "Generate a diagram for a simple web application running on Kubernetes.";

    let agent = ArchitectGPT::new(persona, behavior).await;

    let autogpt = AutoGPT::default()
        .with(agents![agent])
        .build()
        .expect("Failed to build AutoGPT");

    match autogpt.run().await {
        Ok(response) => {
            println!("{}", response);
        }
        Err(err) => {
            eprintln!("Agent error: {:?}", err);
        }
    }
}
```

### 💡 Example Use Cases

Below are a few example patterns to help you integrate agents for various tasks:

#### 🛠️ Backend API Generator

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let persona = "Backend Developer";
    let behavior = "Develop a weather backend apis in Rust using axum.";

    let agent = BackendGPT::new(persona, behavior, "rust").await;

    let autogpt = AutoGPT::default()
        .with(agents![agent])
        .build()
        .expect("Failed to build AutoGPT");

    match autogpt.run().await {
        Ok(response) => {
            println!("{}", response);
        }
        Err(err) => {
            eprintln!("Agent error: {:?}", err);
        }
    }
}
```

#### 🎨 Frontend UI Designer

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let persona = "UX/UI Designer";
    let behavior = "Generate UI for a weather app using React JS.";

    let agent = FrontendGPT::new(persona, behavior, "javascript").await;

    let autogpt = AutoGPT::default()
        .with(agents![agent])
        .build()
        .expect("Failed to build AutoGPT");

    match autogpt.run().await {
        Ok(response) => {
            println!("{}", response);
        }
        Err(err) => {
            eprintln!("Agent error: {:?}", err);
        }
    }
}
```

#### 🧠 Custom General Purpose Agent

```rust
use autogpt::prelude::*;

/// To be compatible with AutoGPT, an agent must implement the `Agent`,
/// `Functions`, and `AsyncFunctions` traits.
/// These traits can be automatically derived using the `Auto` macro.
/// The agent struct must contain at least the following fields.
#[derive(Debug, Default, Auto)]
pub struct CustomAgent {
    pub agent: AgentGPT,
    pub client: ClientType,
}

#[async_trait]
impl Executor for CustomAgent {
    async fn execute<'a>(
        &'a mut self,
        task: &'a mut Task,
        execute: bool,
        browse: bool,
        max_tries: u64,
    ) -> Result<()> {
        // Custom agent logic to interact with `client` (e.g. OpenAI, Gemini, XAI, etc).

        // Use the `generate` method to send the agent's behavior as a prompt
        // to the configured AI client (e.g., OpenAI, Gemini, Claude). This abstracts
        // over the client implementation and returns a model-generated response.
        let behavior = self.agent.behavior().clone();
        let response = self.generate(behavior.as_ref()).await?;

        // (Optional) Store the result in the task or agent state
        self.agent.add_message(Message {
            role: "assistant".into(),
            content: response.clone().into(),
        });

        // (Optional) Store the result in the vector DB (e.g. pinecone)
        let _ = self
            .save_ltm(Message {
                role: "assistant".into(),
                content: response.clone().into(),
            })
            .await;
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let persona = "General Purpose Agent";
    let behavior = "Can do anything.";

    let agent = CustomAgent::new(persona.into(), behavior.into());

    let autogpt = AutoGPT::default()
        .with(agents![agent])
        .build()
        .expect("Failed to build AutoGPT");

    match autogpt.run().await {
        Ok(response) => {
            println!("{}", response);
        }
        Err(err) => {
            eprintln!("Agent error: {:?}", err);
        }
    }
}
```

#### 🤝 Collaborative Multi-Provider Agent (SDK: `col` feature)

Attach a [`CollabPool`] to `AutoGPT` so the orchestrator executes agents
sequentially in round-robin provider order. Each entry in `agents![...]`
corresponds to one entry in the pool's provider list.

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let persona = "Lead Architect";
    let behavior = "Design a microservices architecture for an e-commerce platform.";

    let pool = CollabPool::from_env(persona, behavior, "/tmp/ws", false, false, None, None);

    let agent_a = ArchitectGPT::new(persona, behavior).await;
    let agent_b = ArchitectGPT::new(persona, behavior).await;

    let autogpt = AutoGPT::default()
        .with(agents![agent_a, agent_b])
        .with_collab_pool(pool)
        .build()
        .expect("Failed to build AutoGPT");

    match autogpt.run().await {
        Ok(response) => println!("{}", response),
        Err(err) => eprintln!("Agent error: {:?}", err),
    }
}
```

> [!NOTE]
> **How it works**: `AutoGPT::run()` picks a random starting provider via
> [`CollabSelection::Random`], then executes each agent in the `agents![...]`
> list sequentially mapped to pool providers in round-robin order.
> Enable `CollabSelection::Explicit(name)` to always start from a specific
> provider.

#### 🧠 Agent Metacognition SDK (`mta` feature)

The `mta` feature embeds a [`MetacognitionEngine`] inside every [`AgentGPT`]
that records task outcomes and injects strategy context into prompts.

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let persona = "Research Analyst";
    let behavior = "Summarise research papers.";

    let mut agent = AgentGPT::new_borrowed(persona, behavior);

    agent.record_task_outcome("parse pdf", "success", 0);
    agent.record_task_outcome("extract sections", "failed", 2);

    println!("{}", agent.metacognition_context());
    println!("Should adjust? {}", agent.should_adjust_strategy());
}
```

| Method                                        | Description                                         |
| --------------------------------------------- | --------------------------------------------------- |
| `record_task_outcome(task, outcome, retries)` | Record a task result and generate an insight entry  |
| `metacognition_context()`                     | Returns the LLM-injectable strategy context string  |
| `should_adjust_strategy()`                    | `true` when the engine recommends a strategy change |
| `consecutive_failures()`                      | Number of consecutive failing tasks                 |

The engine triggers automatically inside `GenericAgent` every 3 tasks or
after 2+ consecutive failures, querying the LLM to produce strategy
adjustments. The TUI status bar shows **MetaCognizing** during this phase.

## 🛠️ CLI Usage

The CLI provides a convenient means to interact with the code generation ecosystem. The `autogpt` crate bundles two binaries in a single package:

- `orchgpt` - Launches the orchestrator that manages agents.
- `autogpt` - Launches an agent.

Before utilizing the CLI, you need to **set up environment variables**. These are essential for establishing a secure connection with the orchestrator using the IAC protocol.

### Environment Variables Setup

To configure the CLI and or the SDK environment, follow these steps:

1. **Define Orchestrator Bind Address (Required If Using CLI)**: The orchestrator listens for incoming agent requests over a secure TLS connection. By default, it binds to `0.0.0.0:8443`. You can override this behavior by setting the `ORCHESTRATOR_ADDRESS` environment variable:

   ```sh
   export ORCHESTRATOR_ADDRESS=127.0.0.1:9443
   ```

   This tells the orchestrator to bind to `127.0.0.1` on port `9443` instead of the default.

1. **Define Workspace Path**: GenericGPT defaults to the **current directory** where the CLI is launched as its workspace. Set this to an explicit path when you want generated files scoped to a fixed location:

   ```sh
   export AUTOGPT_WORKSPACE=workspace/
   ```

   For the classic multi-agent workflow (BackendGPT, FrontendGPT, etc.), agents write to subdirectories of the configured root:

   ```sh
   <AUTOGPT_WORKSPACE>/
   ├── architect/
   ├── backend/
   ├── frontend/
   └── designer/
   ```

1. **AI Provider Selection**: You can control which AI client is initialized at runtime using the `AI_PROVIDER` environment variable.
   - `gemini` - Initializes the Gemini client (**requires** the `gem` feature). This is the **default** if `AI_PROVIDER` is not set.
   - `openai` - Initializes the OpenAI client (**requires** the `oai` feature).
   - `anthropic` - Initializes the Anthropic Claude client (**requires** the `cld` feature).
   - `xai` - Initializes the XAI Grok client (**requires** the `xai` feature).
   - `cohere` - Initializes the Cohere client (**requires** the `co` feature).
   - `huggingface` - Initializes the HuggingFace Inference API client (**requires** the `hf` feature).

   ```sh
   # Use Gemini (default, requires `--features gem`)
   export AI_PROVIDER=gemini

   # Use OpenAI (requires `--features oai`)
   export AI_PROVIDER=openai

   # Use Anthropic Claude (requires `--features cld`)
   export AI_PROVIDER=anthropic

   # Use XAI Grok (requires `--features xai`)
   export AI_PROVIDER=xai

   # Use Cohere (requires `--features co`)
   export AI_PROVIDER=cohere

   # Use HuggingFace Inference API (requires `--features hf`)
   export AI_PROVIDER=huggingface
   ```

   Make sure to enable the corresponding Cargo features (`gem`, `oai`, `xai`, `cld`, `co`, `hf`, or `mop`) when building your project.

### 🔀 Mixture of Providers (MoP) Configuration

When using the `--mixture` flag, AutoGPT will attempt to fan out prompts to **every** provider that is compiled in (via feature flags) and has its corresponding API key set in the environment.

Example: If you have `GEMINI_API_KEY` and `OPENAI_API_KEY` set, and build with `--features gem,oai,mop`, running with `--mixture` will automatically use both providers for every query.

1. **API Key Configuration**: Set the API key for your chosen provider:

   ```sh
   # Gemini (default)
   export GEMINI_API_KEY=<your_gemini_api_key>

   # OpenAI
   export OPENAI_API_KEY=<your_openai_api_key>

   # Anthropic Claude
   export ANTHROPIC_API_KEY=<your_anthropic_api_key>

   # XAI Grok
   export XAI_API_KEY=<your_xai_api_key>

   # Cohere
   export COHERE_API_KEY=<your_cohere_api_key>

   # HuggingFace
   export HF_API_KEY=<your_huggingface_api_key>
   ```

   Obtain a Gemini API key from [Google AI Studio](https://aistudio.google.com/app/apikey).

1. **Model Override (Optional)**: Override the default model for any provider using provider-specific env vars or the global fallback:

   ```sh
   export GEMINI_MODEL=gemini-2.5-pro-preview-05-06
   export OPENAI_MODEL=gpt-4o
   ```

1. **DesignerGPT Setup (Optional)**: To enable DesignerGPT, you will need to set up the following environment variable:

   ```sh
   export GETIMG_API_KEY=<your_getimg_api_key>
   ```

   Generate an API key from your [GetImg Dashboard](https://dashboard.getimg.ai/api-keys).

1. **MailerGPT Setup (Optional)**: To enable MailerGPT, in addition to these environment variables, you will need to set up the environment:

   ```sh
   export NYLAS_SYSTEM_TOKEN=<Your_Nylas_System_Token>
   export NYLAS_CLIENT_ID=<Your_Nylas_Client_ID>
   export NYLAS_CLIENT_SECRET=<Your_Nylas_Client_Secret>
   ```

   Follow [this tutorial](NYLAS.md) for a guide on how to obtain these values.

1. **Pinecone Setup (Optional)**: To persist agents memory in a vector database, you will need to set up these environment variables:

   ```sh
   export PINECONE_API_KEY=<Your_Pinecone_API_Key>
   export PINECONE_INDEX_URL=<Your_Pinecone_Index_URL>
   ```

   Follow [this tutorial](PINECONE.md) for a guide on how to obtain these values.

### 🚀 Running the Orchestrator

To launch the orchestrator and start listening for incoming agent connections over TLS, simply run:

```sh
orchgpt
```

### 🧠 Running Agents

#### 🤖 Interactive Mode (Default)

To launch the **GenericGPT interactive shell**, simply run `autogpt` with no arguments:

```sh
autogpt
```

This opens a conversational AI shell where you can:

- Type any prompt to get an immediate response from the active agent.
- Switch providers at runtime with `/provider`.
- Browse and switch models with `/models`.
- List and resume past sessions with `/sessions`.
- Check current status with `/status`.
- Press `ESC` to interrupt a running generation.
- Type `exit` or `quit` to save the session and close.

#### 🤝 Collaborative Multi-Provider Mode (`--collab`, requires `col + cli`)

Collab mode spawns one `GenericAgent` per configured provider (all providers
with an API key set in the environment) and distributes the task across them
in round-robin order with per-model fallback:

```sh
# Build with collab support:
cargo run --features "cli,col,gem,oai,xai" --bin autogpt -- --collab

# Or install:
cargo install autogpt --features "cli,col,gem,oai"
autogpt --collab
```

**Provider discovery**: Any provider feature compiled in whose API key env var
is set at runtime is automatically added to the pool (e.g. `GEMINI_API_KEY`,
`OPENAI_API_KEY`, `XAI_API_KEY`).

**Fallback behaviour**:

| Event                             | Action                                        |
| --------------------------------- | --------------------------------------------- |
| Rate-limit / quota error          | Rotate to next model within the same provider |
| All models for provider exhausted | Remove provider from pool, route to next      |
| All providers exhausted           | Log `⚠ All collab providers exhausted.`       |

The TUI logs the active pool and each routing step with `🤝`/`🔀` prefixes.

#### ⚡ Direct Prompt Mode

For a quick one-shot non-interactive prompt:

```sh
autogpt -p "Explain what a Rust lifetime is"
```

#### 🌐 Networking Mode

To connect to an orchestrator and interact with specialized agents (ArchitectGPT, BackendGPT, etc.), first start the orchestrator, then run:

```sh
autogpt --net
```

This command connects to the orchestrator over a secure TLS connection using the configured address.

Once connected, you can interact with agents using:

```sh
/<agent_name> <action> <input> | <language>
```

For example, to instruct the orchestrator to **create** a new agent, send a command like:

```sh
/arch create "fastapi app" | python
```

This will send a message to the orchestrator with:

- `msg_type`: `"create"`
- `to`: `"ArchitectGPT"`
- `payload_json`: `"fastapi app"`
- `language`: `"python"`

The orchestrator will then initialize and register an `ArchitectGPT` agent ready to perform tasks.

You can also run OrchGPT CLI using Docker:

```sh
docker run -i \
  -e GEMINI_API_KEY=<your_gemini_api_key> \
  -e PINECONE_API_KEY=<Your_Pinecone_API_Key> \
  -e PINECONE_INDEX_URL=<Your_Pinecone_Index_URL> \
  -t kevinrsdev/orchgpt
```

You can also run AutoGPT CLI using Docker:

```sh
docker run -i \
  -e GEMINI_API_KEY=<your_gemini_api_key> \
  -e PINECONE_API_KEY=<Your_Pinecone_API_Key> \
  -e PINECONE_INDEX_URL=<Your_Pinecone_Index_URL> \
  --rm --name autogpt kevinrsdev/autogpt
```

---

© 2026 Wise AI Foundation
