# 📦 Installation

Welcome! AutoGPT offers seamless integration with both Cargo and Docker for easy installation and usage.

## 📦 Install From Registry

### ⚓ Using Cargo

To install AutoGPT CLI via Cargo, execute the following command:

```sh
cargo install autogpt --all-features
```

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
git clone https://github.com/kevin-rs/autogpt.git
```

Navigate to the autogpt directory:

```sh
cd autogpt
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

### 🐳 Using Docker

Build the `orchgpt` Docker container:

```sh
docker build -f Dockerfile.orchgpt -t orchgpt .
```

Build the `autogpt` Docker container:

```sh
docker build -f Dockerfile.autogpt -t autogpt .
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

```bash
docker compose up --build
```

This will:

- Build both `autogpt` and `orchgpt` images using their respective Dockerfiles.
- Create and start the containers.
- Allow `autogpt` to communicate with `orchgpt`.

---

## 🔐 TLS Certificate Setup (Required If Using CLI or Compose V2)

Before running the AutoGPT CLI or using the SDK, you **must set up a local TLS certificate**. This certificate is required to establish secure communication between the CLI and the orchestrator.

To generate a **self-signed TLS certificate**, run the following command in your terminal:

```sh
openssl req -x509 -newkey rsa:2048 -nodes -keyout key.pem -out cert.pem -days 365 \
   -subj "/CN=localhost" \
   -addext "subjectAltName=DNS:localhost" \
   -addext "basicConstraints=critical,CA:FALSE"
```

- `-x509`: Generate a self-signed certificate.
- `-newkey rsa:2048`: Create a new RSA private key (2048-bit).
- `-nodes`: Skip the passphrase for the private key.
- `-keyout key.pem`: Output file for the private key.
- `-out cert.pem`: Output file for the certificate.
- `-days 365`: Validity of the certificate (1 year).
- `-subj "/CN=localhost"`: Set the Common Name to `localhost`.
- `-addext "subjectAltName=DNS:localhost"`: Specify the Subject Alternative Name.
- `-addext "basicConstraints=critical,CA:FALSE"`: Restrict the certificate from acting as a Certificate Authority.

The generated `cert.pem` and `key.pem` file must be made available in a **certs** directory in the root project.

---

## 🧰 SDK Usage

The SDK offers a simple and flexible API for building and running intelligent agents in your applications. Before getting started, **make sure to configure the required environment variables**. For detailed setup, refer to the [Environment Variables Setup](#environment-variables-setup) section.

Once the environment is ready, you can quickly spin up an agent like so:

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let position = "Lead UX/UI Designer";

    let prompt = "Generate a diagram for a simple web application running on Kubernetes.";

    let agent = ArchitectGPT::new(prompt, position).await;

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
    let position = "Backend Developer";

    let prompt = "Develop a weather backend apis in Rust using axum.";

    let agent = BackendGPT::new(prompt, position, "rust").await;

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
    let position = "UX/UI Designer";

    let prompt = "Generate UI for a weather app using React JS.";

    let agent = FrontendGPT::new(prompt, position, "javascript").await;

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
    objective: Cow<'static, str>,
    position: Cow<'static, str>,
    status: Status,
    agent: AgentGPT,
    client: ClientType,
    memory: Vec<Communication>,
}

#[async_trait]
impl AgentExecutor for CustomAgent {
    async fn execute<'a>(
        &'a mut self,
        tasks: &'a mut Task,
        execute: bool,
        browse: bool,
        max_tries: u64,
    ) -> Result<()> {
        // Custom agent logic to interact with `client` (e.g. OpenAI, Gemini, XAI, etc).
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let position = "General Purpose Agent";

    let prompt = "Can do anything.";

    let agent = CustomAgent::new(prompt.into(), position.into());

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

## 🛠️ CLI Usage

The CLI provides a convenient means to interact with the code generation ecosystem. The `autogpt` crate bundles two binaries in a single package:

- `orchgpt` - Launches the orchestrator that manages agents.
- `autogpt` - Launches an agent.

Before utilizing the CLI, you need to **set up TLS**. To do so, make sure you've created `cert.pem` and `key.pem` under the **certs** directory using the previous command. These are essential for establishing a secure connection with the orchestrator.

### Environment Variables Setup

To configure the CLI and or the SDK environment, follow these steps:

1. **Define Orchestrator Bind Address (Required If Using CLI)**: The orchestrator listens for incoming agent requests over a secure TLS connection. By default, it binds to `0.0.0.0:8443`. You can override this behavior by setting the `ORCHESTRATOR_ADDRESS` environment variable:

   ```sh
   export ORCHESTRATOR_ADDRESS=127.0.0.1:9443
   ```

   This tells the orchestrator to bind to `127.0.0.1` on port `9443` instead of the default.

1. **Define Workspace Path**: Set up the paths for designer, backend, frontend, and architect workspaces by setting the following environment variable:

   ```sh
   export AUTOGPT_WORKSPACE=workspace/
   ```

   This variable guide the agents on where to generate the code within your project structure.

1. **AI Provider Selection**: You can control which AI client is initialized at runtime using the `AI_PROVIDER` environment variable.

   - `xai` - Initializes the XAI Grok client (**requires** the `xai` feature).
   - `openai` - Initializes the OpenAI client (**requires** the `oai` feature).
   - `anthropic` - Initializes the Anthropic Claude client (**requires** the `cld` feature).
   - `gemini` - Initializes the Gemini client (**requires** the `gem` feature). This is the **default** if `AI_PROVIDER` is not set.

   ```sh
   # Use OpenAI (requires `--features oai`)
   export AI_PROVIDER=openai

   # Use Gemini (requires `--features gem`)
   export AI_PROVIDER=gemini

   # Use Anthropic Claude (requires `--features cld`)
   export AI_PROVIDER=anthropic

   # Use XAI Grok (requires `--features xai`)
   export AI_PROVIDER=xai
   ```

   Make sure to enable the corresponding Cargo features (`oai`, `xai`, `cld`, or `gem`) when building your project.

1. **API Key Configuration**: Additionally, you need to set up the Gemini API key by setting the following environment variable:

   ```sh
   export GEMINI_API_KEY=<your_gemini_api_key>
   ```

   To obtain your API key, navigate to [Google AI Studio](https://aistudio.google.com/app/apikey) and generate it there. This key allows autogpt to communicate with Gemini API.

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

To start an agent and establish a connection with the orchestrator (either locally or on a remote machine), run:

```sh
autogpt
```

This command launches the agent and connects it to the orchestrator over a secure TLS connection using the configured address.

Once the agent is running, you can interact with it using simple command syntax:

```sh
/<agent_name> <action> <input> | <language>
```

For example, to instruct the orchestrator to **create** a new agent, send a command like:

```sh
/ArchitectGPT create "fastapi app" | python
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
  -t orchgpt:latest
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
