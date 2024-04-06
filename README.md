# 🤖 AutoGPT

<div align="center">

[![CircleCI](https://dl.circleci.com/status-badge/img/gh/kevin-rs/autogpt/tree/main.svg?style=svg&circle-token=CCIPRJ_PifnErxs6Ze2XWpjmUeRV1_4e84825e0f6a366716a77c2dbbe93c3bd3e507fa)](https://dl.circleci.com/status-badge/redirect/gh/kevin-rs/autogpt/tree/main)
[![Github](https://img.shields.io/badge/launch-Github-181717.svg?logo=github&logoColor=white)](./examples/basic.ipynb)
[![Binder](https://mybinder.org/badge_logo.svg)](https://mybinder.org/v2/gh/kevin-rs/autogpt/main?filepath=examples/basic.ipynb)
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/kevin-rs/autogpt/blob/main/examples/basic.ipynb)

![banner](https://github.com/kevin-rs/kevin/assets/62179149/8b54dea8-2231-4509-8c18-10ec414578d2)

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

<video src="https://github.com/kevin-rs/kevin/assets/62179149/ba6f7204-849e-4b89-ae92-5b7faa0be68a"></video>

</div>

AutoGPT is a pure rust framework that simplifies AI agent creation and management for various tasks. Its remarkable speed and versatility are complemented by a mesh of built-in interconnected GPTs, ensuring exceptional performance and adaptability.

---

## 🚀 Features

- **Agent Creation**: Easily create different types of agents tailored to specific tasks.
- **Task Management**: Efficiently manage tasks and distribute them among agents.
- **Extensible**: Extend functionality by adding new agent types and task handling capabilities.
- **CLI Interface**: Command-line interface for seamless interaction with the framework.
- **SDK Integration**: Software development kit for integrating AutoGPT into existing projects.

---

## 📦 Installation

Please refer to [our tutorial](INSTALLATION.md) for guidance on installing, running, and/or building the CLI from source using either Cargo or Docker.

> [!NOTE]
> For optimal performance and compatibility, we strongly advise utilizing a Linux operating system to install this CLI.

## 🔄 Workflow

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
|    GPT     |       |      GPT        |<----->|    GPT      |       |  GPT        |
|            |       |                 |       |  (Optional) |       |             |
+--+---------+       +-----------------+       +-------------+       +-------------+
   |                          |                       |                       |
   v                          v                       v                       v
(Backend Logic)        (Frontend Logic)         (Designer Logic)        (Architect Logic)
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
  
- 🚀 **Initialization**: AutoGPT initializes based on the user's input, creating essential components such as the ManagerGPT and individual agent instances (ArchitectGPT, BackendGPT, FrontendGPT).
  
- 🛠️ **Agent Configuration**: Each agent is configured with its unique objectives and capabilities, aligning them with the project's defined goals. This configuration ensures that agents contribute effectively to the project's objectives.
  
- 📋 **Task Allocation**: ManagerGPT distributes tasks among agents considering their capabilities and project requirements.
  
- ⚙️ **Task Execution**: Agents execute tasks asynchronously, leveraging their specialized functionalities.
  
- 🔄 **Feedback Loop**: Continuous feedback updates users on project progress and addresses issues.
  
## 🤖 Available Agents

At the current release, Autogpt consists of 6 built-in specialized autonomous AI agents ready to assist you in bringing your ideas to life!
Refer to [our guide](AGENTS.md) to learn more about how the built-in agents work.

## 📌 Examples

Your can refer to [our examples](EXAMPLES.md) for guidance on how to use the cli in a jupyter environment.

## 📚 Documentation

For detailed usage instructions and API documentation, refer to the [AutoGPT Documentation](https://docs.rs/autogpt).

## 🤝 Contributing

Contributions are welcome! See the [Contribution Guidelines](CONTRIBUTING.md) for more information on how to get started.

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---