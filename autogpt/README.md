<div align="center">

# 🤖 AutoGPT

[![Work In Progress](https://img.shields.io/badge/Work%20In%20Progress-red)](https://github.com/wiseaidev)
[![made-with-rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-brightgreen.svg)](LICENSE)
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)](https://github.com/wiseaidev)
[![Jupyter Notebook](https://img.shields.io/badge/Jupyter-Notebook-blue.svg?logo=Jupyter&logoColor=orange)](https://jupyter.org/)

[![Share On Reddit](https://img.shields.io/badge/share%20on-reddit-red?logo=reddit)](https://reddit.com/submit?url=https://github.com/kevin-rs/autogpt&title=World%27s%20First%2C%20Multimodal%2C%20Zero%20Shot%2C%20Most%20General%2C%20Most%20Capable%2C%20Blazingly%20Fast%2C%20and%20Extremely%20Flexible%20Pure%20Rust%20AI%20Agentic%20Framework.)
[![Share On Ycombinator](https://img.shields.io/badge/share%20on-hacker%20news-orange?logo=ycombinator)](https://news.ycombinator.com/submitlink?u=https://github.com/kevin-rs/autogpt&t=World%27s%20First%2C%20Multimodal%2C%20Zero%20Shot%2C%20Most%20General%2C%20Most%20Capable%2C%20Blazingly%20Fast%2C%20and%20Extremely%20Flexible%20Pure%20Rust%20AI%20Agentic%20Framework.)
[![Share On X](https://img.shields.io/badge/share%20on-X-03A9F4?logo=x)](https://twitter.com/share?url=https://github.com/kevin-rs/autogpt&text=World%27s%20First%2C%20Multimodal%2C%20Zero%20Shot%2C%20Most%20General%2C%20Most%20Capable%2C%20Blazingly%20Fast%2C%20and%20Extremely%20Flexible%20Pure%20Rust%20AI%20Agentic%20Framework.)
[![Share On Meta](https://img.shields.io/badge/share%20on-meta-1976D2?logo=meta)](https://www.facebook.com/sharer/sharer.php?u=https://github.com/kevin-rs/autogpt)
[![Share On Linkedin](https://img.shields.io/badge/share%20on-linkedin-3949AB?logo=linkedin)](https://www.linkedin.com/shareArticle?url=https://github.com/kevin-rs/autogpt&title=World%27s%20First%2C%20Multimodal%2C%20Zero%20Shot%2C%20Most%20General%2C%20Most%20Capable%2C%20Blazingly%20Fast%2C%20and%20Extremely%20Flexible%20Pure%20Rust%20AI%20Agentic%20Framework.)

[![CircleCI](https://dl.circleci.com/status-badge/img/gh/kevin-rs/autogpt/tree/main.svg?style=svg&circle-token=CCIPRJ_PifnErxs6Ze2XWpjmUeRV1_4e84825e0f6a366716a77c2dbbe93c3bd3e507fa)](https://dl.circleci.com/status-badge/redirect/gh/kevin-rs/autogpt/tree/main)
[![Crates.io Downloads](https://img.shields.io/crates/d/autogpt)](https://crates.io/crates/autogpt)
[![Github](https://img.shields.io/badge/launch-Github-181717.svg?logo=github&logoColor=white)](./examples/basic.ipynb)
[![Binder](https://mybinder.org/badge_logo.svg)](https://mybinder.org/v2/gh/kevin-rs/autogpt/main?filepath=examples/basic.ipynb)
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/kevin-rs/autogpt/blob/main/examples/basic.ipynb)

![banner](https://github.com/user-attachments/assets/c642e17a-f164-44b5-9cd1-bc1711cebbbf)

| 🐧 Linux `(Recommended)` | 🪟 Windows | 🐋 | 🐋 |
| :------: | :--------: | :--------: | :--------: |
| [![Crates.io Downloads](https://img.shields.io/crates/d/autogpt)](https://crates.io/crates/autogpt) | [![Crates.io Downloads](https://img.shields.io/crates/d/autogpt)](https://crates.io/crates/autogpt) | [![Docker](https://img.shields.io/docker/pulls/kevinrsdev/autogpt.svg)](https://hub.docker.com/r/kevinrsdev/autogpt) | [![Docker](https://img.shields.io/docker/pulls/kevinrsdev/orchgpt.svg)](https://hub.docker.com/r/kevinrsdev/orchgpt) |
| ![linux-demo](https://raw.githubusercontent.com/kevin-rs/autogpt/refs/heads/main/assets/linux.png) | ![windows-demo](https://raw.githubusercontent.com/kevin-rs/autogpt/refs/heads/main/assets/windows.png) | - | - |
| Method 1: [Download Executable File](https://github.com/kevin-rs/autogpt/releases/download/v0.1.13/autogpt) | [Download `.exe` File](https://github.com/kevin-rs/autogpt/releases/download/v0.1.13/autogpt.exe) | - | - |
| Method 2: `cargo install autogpt --all-features` | `cargo install autogpt --all-features` |  `docker pull kevinrsdev/autogpt:0.1.13` | `docker pull kevinrsdev/orchgpt:0.1.13` |
| [**Set Environment Variables**](https://github.com/kevin-rs/autogpt/blob/main/INSTALLATION.md#environment-variables-setup) | [**Set Environment Variables**](https://github.com/kevin-rs/autogpt/blob/main/INSTALLATION.md#environment-variables-setup) | [**Set Environment Variables**](https://github.com/kevin-rs/autogpt/blob/main/INSTALLATION.md#-using-docker) | [**Set Environment Variables**](https://github.com/kevin-rs/autogpt/blob/main/INSTALLATION.md#-using-docker) |
| `autogpt -h` <br> `orchgpt -h` | `autogpt.exe -h` |  `docker run kevinrsdev/autogpt:0.1.13 -h` | `docker run kevinrsdev/orchgpt:0.1.13 -h` |

<video src="https://github.com/user-attachments/assets/ecd82549-a48f-49c2-b751-23f74820bf3d"></video>

</div>

AutoGPT is a pure rust framework that simplifies AI agent creation and management for various tasks. Its remarkable speed and versatility are complemented by a mesh of built-in interconnected GPTs, ensuring exceptional performance and adaptability.

## 🧠 Framework Overview

### ⚙️ Agent Core Architecture

AutoGPT agents are modular and autonomous, built from composable components:

- 🔌 **Tools & Sensors**: Interface with the real world via actions (e.g., file I/O, APIs) and perception (e.g., audio, video, data).
- 🧠 **Memory & Knowledge**: Combines long-term vector memory with structured knowledge bases for reasoning and recall.
- 🧭 **Planner & Goals**: Breaks down complex tasks into subgoals and tracks progress dynamically.
- 🧍 **Persona & Capabilities**: Customizable behavior profiles and access controls define how agents act.
- 🧑‍🤝‍🧑 **Collaboration**: Agents can delegate, swarm, or work in teams with other agents.
- 🪞 **Self-Reflection**: Introspection module to debug, adapt, or evolve internal strategies.
- 🔄 **Context Management**: Manages active memory (context window) for ongoing tasks and conversations.
- 📅 **Scheduler**: Time-based or reactive triggers for agent actions.

### 🚀 Developer Features

AutoGPT is designed for flexibility, integration, and scalability:

- 🧪 **Custom Agent Creation**: Build tailored agents for different roles or domains.
- 📋 **Task Orchestration**: Manage and distribute tasks across agents efficiently.
- 🧱 **Extensibility**: Add new tools, behaviors, or agent types with ease.
- 💻 **CLI Tools**: Command-line interface for rapid experimentation and control.
- 🧰 **SDK Support**: Embed AutoGPT into existing projects or systems seamlessly.

## 📦 Installation

Please refer to [our tutorial](INSTALLATION.md) for guidance on installing, running, and/or building the CLI from source using either Cargo or Docker.

> [!NOTE]
> For optimal performance and compatibility, we strongly advise utilizing a Linux operating system to install this CLI.

## 🔄 Workflow

AutoGPT supports two modes of operation, enabling both standalone and distributed use cases:

### 1. 🧠 Agentic Networkless Mode (Standalone)

<video src="https://github.com/user-attachments/assets/7d47b1d8-b2f2-4d23-a1f4-da926e425330"></video>

In this mode, the user runs an individual `autogpt` agent directly via a subcommand (e.g., `autogpt arch`). Each agent operates independently without needing a networked orchestrator.

```sh
                       +------------------------------------+
                       |                User                |
                       |             Provides               |
                       |          Project Prompt            |
                       +------------------+-----------------+
                                          |
                                          v
                       +------------------+-----------------+
                       |               ManagerGPT           |
                       |            Distributes Tasks       |
                       |          to Backend, Frontend,     |
                       |           Designer, Architect      |
                       +------------------+-----------------+
                                          |
                                          v
   +--------------------------+-----------+----------+----------------------+
   |                          |                      |                      |
   |                          v                      v                      v
+--+---------+       +--------+--------+       +-----+-------+       +-----+-------+
|  Backend   |       |    Frontend     |       |  Designer   |       |  Architect  |
|    GPT     |       |      GPT        |  ...  |    GPT      |       |  GPT        |
|            |       |                 |       |  (Optional) |       |             |
+--+---------+       +-----------------+       +-------------+       +-------------+
   |                          |                       |                       |
   v                          v                       v                       v
(Backend Logic)        (Frontend Logic)   ...   (Designer Logic)        (Architect Logic)
   |                          |                       |                       |
   +--------------------------+----------+------------+-----------------------+
                                         |
                                         v
                      +------------------+-----------------+
                      |               ManagerGPT           |
                      |       Collects and Consolidates    |
                      |        Results from Agents         |
                      +------------------+-----------------+
                                         |
                                         v
                      +------------------+-----------------+
                      |                User                |
                      |            Receives Final          |
                      |             Output from            |
                      |            ManagerGPT              |
                      +------------------------------------+
```

- ✍️ **User Input**: Provide a project's goal (e.g. "Develop a full stack app that fetches today's weather. Use the axum web framework for the backend and the Yew rust framework for the frontend.").
- 🚀 **Initialization**: AutoGPT initializes based on the user's input, creating essential components such as the `ManagerGPT` and individual agent instances (ArchitectGPT, BackendGPT, FrontendGPT).
- 🛠️ **Agent Configuration**: Each agent is configured with its unique objectives and capabilities, aligning them with the project's defined goals. This configuration ensures that agents contribute effectively to the project's objectives.
- 📋 **Task Allocation**: ManagerGPT distributes tasks among agents considering their capabilities and project requirements.
- ⚙️ **Task Execution**: Agents execute tasks asynchronously, leveraging their specialized functionalities.
- 🔄 **Feedback Loop**: Continuous feedback updates users on project progress and addresses issues.

### 2. 🌐 Agentic Networking Mode (Orchestrated)

<video src="https://github.com/user-attachments/assets/ecd82549-a48f-49c2-b751-23f74820bf3d"></video>

In networking mode, `autogpt` connects to an external orchestrator (`orchgpt`) over a secure TLS-encrypted TCP channel. This orchestrator manages agent lifecycles, routes commands, and enables rich inter-agent collaboration using a unified protocol.

AutoGPT introduces a novel and scalable communication protocol called [`IAC`](IAC.md) (Inter/Intra-Agent Communication), enabling seamless and secure interactions between agents and orchestrators, inspired by [operating system IPC mechanisms](https://en.wikipedia.org/wiki/Inter-process_communication).

In networking mode, AutoGPT utilizes a layered architecture:

```sh
                       +------------------------------------+
                       |                User                |
                       |         Sends Prompt via CLI       |
                       +------------------+-----------------+
                                          |
                                          v
                            TLS + Protobuf over TCP to:
                       +------------------+-----------------+
                       |             Orchestrator           |
                       |     Receives and Routes Commands   |
                       +-----------+----------+-------------+
                                   |          |
     +-----------------------------+          +----------------------------+
     |                                                                     |
     v                                                                     v
+--------------------+                                         +--------------------+
|   ArchitectGPT     |<---------------- IAC ------------------>|    ManagerGPT      |
+--------------------+                                         +--------------------+
   |                        Agent Layer:                                   |
   |          (BackendGPT, FrontendGPT, DesignerGPT)                       |
   +-------------------------------------+---------------------------------+
                                         |
                                         v
                               Task Execution & Collection
                                         |
                                         v
                           +---------------------------+
                           |           User            |
                           |     Receives Final Output |
                           +---------------------------+
```

All communication happens securely over **TLS + TCP**, with messages encoded in **Protocol Buffers (protobuf)** for efficiency and structure.

1. User Input: The user provides a project prompt like:

   ```sh
   /architect create "fastapi app" | python
   ```

   This is securely sent to the Orchestrator over TLS.

1. Initialization: The Orchestrator parses the command and initializes the appropriate agent (e.g., `ArchitectGPT`).

1. Agent Configuration: Each agent is instantiated with its specialized goals:

   - **ArchitectGPT**: Plans system structure
   - **BackendGPT**: Generates backend logic
   - **FrontendGPT**: Builds frontend UI
   - **DesignerGPT**: Handles design

1. Task Allocation: `ManagerGPT` dynamically assigns subtasks to agents using the IAC protocol. It determines which agent should perform what based on capabilities and the original user goal.

1. Task Execution: Agents execute their tasks, communicate with their subprocesses or other agents via IAC (inter/intra communication), and push updates or results back to the orchestrator.

1. Feedback Loop: Throughout execution, agents return status reports. The `ManagerGPT` collects all output, and the Orchestrator sends it back to the user.

## 🤖 Available Agents

At the current release, Autogpt consists of 8 built-in specialized autonomous AI agents ready to assist you in bringing your ideas to life!
Refer to [our guide](AGENTS.md) to learn more about how the built-in agents work.

## 📌 Examples

Your can refer to [our examples](EXAMPLES.md) for guidance on how to use the cli in a jupyter environment.

## 📚 Documentation

For detailed usage instructions and API documentation, refer to the [AutoGPT Documentation](https://docs.rs/autogpt).

## 🤝 Contributing

Contributions are welcome! See the [Contribution Guidelines](CONTRIBUTING.md) for more information on how to get started.

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
