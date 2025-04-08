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
docker run -it -e GEMINI_API_KEY=<your_gemini_api_key> --rm --name autogpt kevinrsdev/autogpt:0.0.1
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

To run AutoGPT CLI via Cargo, execute:

```sh
cargo run --all-features
```

### 🐳 Using Docker

Build the Docker container:

```sh
docker build -t autogpt .
```

Run the container:

```sh
docker run -i -e GEMINI_API_KEY=<your_gemini_api_key> -t autogpt:latest
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

---

## 🛠️ CLI Usage

The CLI provides a convenient means to interact with the code generation ecosystem. Before utilizing the CLI and or the SDK, your need to set up the necessary environment variables.

### Environment Variables Setup

To configure the CLI and or the SDK environment, follow these steps:

1. **Define Workspace Path**: Set up the paths for designer, backend, frontend, and architect workspaces by setting the following environment variable:

   ```sh
   export AUTOGPT_WORKSPACE=workspace/
   ```

   This variable guide the agents on where to generate the code within your project structure.

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

### Running Agents

Execute agents to perform tasks using the `run` command:

```sh
autogpt
```

You can also run AutoGPT CLI using Docker:

```sh
docker run -it -e GEMINI_API_KEY=<your_gemini_api_key> --rm --name autogpt kevinrsdev/autogpt:0.0.1
```

---
