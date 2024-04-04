# 🤖 AutoGPT

<div align="center">

[![Work In Progress](https://img.shields.io/badge/Work%20In%20Progress-red)](https://github.com/wiseaidev)
[![made-with-rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Rust](https://img.shields.io/badge/Rust-1.75.0%2B-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-brightgreen.svg)](LICENSE)
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)](https://github.com/wiseaidev)
[![Jupyter Notebook](https://img.shields.io/badge/Jupyter-Notebook-blue.svg?logo=Jupyter&logoColor=orange)](https://jupyter.org/)

[![Share On Reddit](https://img.shields.io/badge/share%20on-reddit-red?logo=reddit)](https://reddit.com/submit?url=https://github.com/kevin-rs/autogpt&title=World%27s%20First%2C%20Multimodal%2C%20Zero%20Shot%2C%20Most%20General%2C%20Most%20Capable%2C%20Blazingly%20Fast%2C%20and%20Extremely%20Flexible%20Pure%20Rust%20AI%20Agentic%20Framework.)
[![Share On Ycombinator](https://img.shields.io/badge/share%20on-hacker%20news-orange?logo=ycombinator)](https://news.ycombinator.com/submitlink?u=https://github.com/kevin-rs/autogpt&t=World%27s%20First%2C%20Multimodal%2C%20Zero%20Shot%2C%20Most%20General%2C%20Most%20Capable%2C%20Blazingly%20Fast%2C%20and%20Extremely%20Flexible%20Pure%20Rust%20AI%20Agentic%20Framework.)
[![Share On X](https://img.shields.io/badge/share%20on-X-03A9F4?logo=x)](https://twitter.com/share?url=https://github.com/kevin-rs/autogpt&text=World%27s%20First%2C%20Multimodal%2C%20Zero%20Shot%2C%20Most%20General%2C%20Most%20Capable%2C%20Blazingly%20Fast%2C%20and%20Extremely%20Flexible%20Pure%20Rust%20AI%20Agentic%20Framework.)
[![Share On Meta](https://img.shields.io/badge/share%20on-meta-1976D2?logo=meta)](https://www.facebook.com/sharer/sharer.php?u=https://github.com/kevin-rs/autogpt)
[![Share On Linkedin](https://img.shields.io/badge/share%20on-linkedin-3949AB?logo=linkedin)](https://www.linkedin.com/shareArticle?url=https://github.com/kevin-rs/autogpt&title=World%27s%20First%2C%20Multimodal%2C%20Zero%20Shot%2C%20Most%20General%2C%20Most%20Capable%2C%20Blazingly%20Fast%2C%20and%20Extremely%20Flexible%20Pure%20Rust%20AI%20Agentic%20Framework.)

![banner](https://github.com/kevin-rs/kevin/assets/62179149/91c83cbe-07b5-415e-bede-fec973a09d03)

</div>

AutoGPT is a groundbreaking framework that lets you easily create and manage agents for different jobs. It's blazingly fast and can handle lots of tasks. With AutoGPT, you can automate things quickly and efficiently.

## 🚀 Features

- **Agent Creation**: Easily create different types of agents tailored to specific tasks.
- **Task Management**: Efficiently manage tasks and distribute them among agents.
- **Extensible**: Extend functionality by adding new agent types and task handling capabilities.
- **CLI Interface**: Command-line interface for seamless interaction with the framework.
- **SDK Integration**: Software development kit for integrating AutoGPT into existing projects.

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
|            |       |                 |       |             |       |             |
+--+---------+       +-----------------+       +-------------+       +-------------+
   |                          |                       |                       |
   v                          v                       v                       v
Executes Assigned Tasks     Executes Assigned Tasks      Executes Assigned Tasks
  (Backend Logic)          (Frontend Logic)                  (Architect Logic)
   |                                                  |                       |
   +-------------------------------------+------------+-----------------------+
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
  
## 📦 Installation

You can install AutoGPT via Cargo, the Rust package manager:

```bash
cargo install autogpt
```

## 🛠️ CLI Usage

### Running Agents

Execute agents to perform tasks using the `run-agent` command:

```bash
autogpt run <agent-name>
```

## 🤖 Available Agents

![agent](https://github.com/kevin-rs/kevin/assets/62179149/abfb7e37-b1d0-45ec-916e-dc3032eafdb3)

Autogpt consists of 5 built-in specialized autonomous AI agents ready to assist you in bringing your ideas to life!

### 1. 🎩 ManagerGPT

ManagerGPT serves as the orchestrator of your project, directing the other agents to execute tasks based on your input. When you provide a project prompt, ManagerGPT divides it into tasks for BackendGPT, FrontendGPT, DesignerGPT, and ArchitectGPT.

#### How ManagerGPT Works?

Let's say you want to develop a full-stack app that fetches today's weather in Python using FastAPI. ManagerGPT will break down this task into specific steps for each agent:

- For ArchitectGPT: ManagerGPT will instruct ArchitectGPT to design the overall structure of the application, including backend and frontend components, using Python and FastAPI.
- For DesignerGPT: ManagerGPT will guide DesignerGPT to create a user-friendly interface for displaying the weather forecast.
- For BackendGPT: ManagerGPT will assign BackendGPT to implement the backend logic using FastAPI, fetching weather data from external sources.
- For FrontendGPT: ManagerGPT will direct FrontendGPT to develop the frontend interface for users to interact with and visualize the weather data.

---

### 2. 👷‍♀️ ArchitectGPT

ArchitectGPT is responsible for designing the overall structure and architecture of your application. ArchitectGPT will create the foundation upon which your app will be built.

#### How ArchitectGPT Works

Upon receiving instructions from ManagerGPT, ArchitectGPT will:

- Determine the technologies and frameworks needed to realize the project goals, such as Python and FastAPI.
- Design the data flow and communication between backend and frontend components to ensure seamless operation.
- Define the project's scope and establish a roadmap for development, breaking down tasks into manageable steps.

---

### 3. 🎨 DesignerGPT

DesignerGPT transforms ideas into visually stunning designs. Whether it's crafting sleek user interfaces or designing captivating user experiences, DesignerGPT brings your project to life with style.

#### How DesignerGPT Works

When tasked by ManagerGPT, DesignerGPT will:

- Create mockups and wireframes of the application's interface, ensuring a user-friendly and intuitive design.
- Select colors, fonts, and layouts that align with your project's branding and aesthetic.
- TODO: Collaborate with other agents to integrate design elements seamlessly into the final product.

---

### 4. ⚙️ BackendGPT

BackendGPT handles all things related to server-side logic and data processing. From database management to API integration, BackendGPT ensures that your application's backend is robust and efficient.

#### How BackendGPT Works

Upon receiving instructions from ManagerGPT, BackendGPT will:

- Develop the backend infrastructure using FastAPI, implementing endpoints for retrieving weather data and handling user requests.
- Integrate external APIs or services to fetch real-time weather information.
- Ensure data security and integrity, implementing authentication and authorization mechanisms as needed.

---

### 5. 🖥️ FrontendGPT

FrontendGPT will craft engaging and interactive experiences for your application's users. With a keen eye for design and a knack for coding, FrontendGPT brings your designs to life in the browser.

#### How FrontendGPT Works

When prompted by ManagerGPT, FrontendGPT will:

- Develop the frontend interface using modern web technologies such as HTML, CSS, and JavaScript, complementing the backend's functionality.
- Implement responsive design principles to ensure a seamless experience across devices and screen sizes.
- TODO: Collaborate with DesignerGPT to translate design mockups into code, bringing the application's visual identity to fruition.

With Autogpt's team of specialized agents working together, your project is in capable hands. Simply provide a simple project goal, and let Autogpt handle the rest!

---

## 📚 Documentation

For detailed usage instructions and API documentation, refer to the [AutoGPT Documentation](https://docs.rs/autogpt).

## 🤝 Contributing

Contributions are welcome! See the [Contribution Guidelines](CONTRIBUTING.md) for more information on how to get started.

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
