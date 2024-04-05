# 🤖 AutoGPT

<div align="center">

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

![banner](https://github.com/kevin-rs/kevin/assets/62179149/8b54dea8-2231-4509-8c18-10ec414578d2)

</div>

AutoGPT is an innovative framework that simplifies AI agent creation and management for various tasks. Its remarkable speed and versatility are complemented by a mesh of interconnected GPTs, ensuring exceptional performance and adaptability.

---

## 🚀 Features

- **Agent Creation**: Easily create different types of agents tailored to specific tasks.
- **Task Management**: Efficiently manage tasks and distribute them among agents.
- **Extensible**: Extend functionality by adding new agent types and task handling capabilities.
- **CLI Interface**: Command-line interface for seamless interaction with the framework.
- **SDK Integration**: Software development kit for integrating AutoGPT into existing projects.

---

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
Executes Assigned Tasks     Executes Assigned Tasks           Executes Assigned Tasks
  (Backend Logic)          (Frontend Logic)                        (Architect Logic)
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
  
---

## 📦 Installation

You can install AutoGPT via Cargo, the Rust package manager:

```bash
cargo install autogpt
```

---

## 🛠️ CLI Usage

The Command Line Interface (CLI) provides a convenient means to interact with the code generation system. Before utilizing the CLI effectively, it's essential to set up the necessary environment variables for seamless operation.

### Environment Variables Setup

To configure the CLI and or the SDK environment, follow these steps:

1. Define Workspace Path: Set up the paths for designer, backend, frontend, and architect workspaces by setting the following environment variable:
   
   ```sh
   export AUTOGPT_WORKSPACE=workspace/
   ```

   This variable guide the agents on where to generate the code within your project structure.

1. API Key Configuration: Additionally, you need to set up the Gemini API key by setting the following environment variable:

   ```sh
   export GEMINI_API_KEY=<your_gemini_api_key>
   ```

   To obtain your API key, navigate to [Google AI Studio](https://aistudio.google.com/app/apikey) and generate it there. This key allows autogpt to communicate with Gemini API.

### Running Agents

Execute agents to perform tasks using the `run` command:

```bash
autogpt run <agent-name>
```

---

## 🤖 Available Agents

At the current release, Autogpt consists of 6 built-in specialized autonomous AI agents ready to assist you in bringing your ideas to life!

### 1. 🎩 ManagerGPT

<img src="https://github.com/kevin-rs/kevin/assets/62179149/fc7fb72b-6f45-4c35-99ff-5c0f8e7d54cf" align="left" alt="manager" width="64" />

ManagerGPT serves as the orchestrator of your project, directing the other agents to execute tasks based on your input. When you provide a project prompt, ManagerGPT divides it into tasks for BackendGPT, FrontendGPT, DesignerGPT, and ArchitectGPT.

#### How ManagerGPT Works?

Let's say you want to develop a full-stack app that fetches today's weather in Python using FastAPI. ManagerGPT simplifies this process by breaking it down into specific tasks for each specialized agent:

- **ArchitectGPT**: ManagerGPT instructs ArchitectGPT to design the application's structure, encompassing both backend and frontend components, utilizing Python and FastAPI:

```sh
[*] "ManagerGPT": Executing task: "Develop a full stack app that fetches today's weather in python using FastAPI."
```

ManagerGPT articulates the project goal and communicates it to the ArchitectGPT through Gemini API, which then generates specific steps for architecting the application:

```sh
[*] "ArchitectGPT": Executing tasks: Tasks { description: "- Design the user interface for the weather app, including input fields for location and a display area for weather details.\n- Implement a function to fetch current weather data from a weather API in Python.\n- Create a FastAPI endpoint that calls the weather fetching function and returns the weather data in JSON format.\n- Integrate the FastAPI endpoint with the front end to display the fetched weather data on the user interface.\n- Handle error scenarios and provide appropriate user feedback.", scope: None, urls: None, frontend_code: None, backend_code: None, api_schema: None }
```

- **DesignerGPT**: ManagerGPT guides DesignerGPT in crafting a user-friendly interface tailored for presenting the weather forecast. 

```sh
[*] "DesignerGPT": Executing tasks: Tasks { description: "- Utilize FastAPI in Python to develop a user interface for the weather app, integrating a location input field and display section for weather data.\n- Step 1: Define the layout and structure of the user interface, ensuring it's user-friendly and visually appealing.\n- Step 2: Implement the location search functionality, enabling users to input their desired location and retrieve relevant weather information.", scope: None, urls: None, frontend_code: None, backend_code: None, api_schema: None }
```

- **BackendGPT**: ManagerGPT assigns BackendGPT to implement the backend logic using FastAPI, responsible for fetching weather data from external sources.

```sh
[*] "BackendGPT": Executing tasks: Tasks { description: "- Using FastAPI in Python, create a backend for a weather application featuring a user interface with a location input field and weather data display section.\n- Step 1: Design and develop the user interface, ensuring it's user-friendly and visually appealing.\n- Step 2: Implement the location search functionality, enabling users to input their desired location and retrieve relevant weather information.", scope: None, urls: None, frontend_code: None, backend_code: None, api_schema: None }
```

- **FrontendGPT**: ManagerGPT directs FrontendGPT to develop the frontend interface, enabling users to interact with and visualize the weather data.

```sh
[*] "FrontendGPT": Executing tasks: Tasks { description: "- Using FastAPI in Python, create a user interface for a weather application.\n- Step 1: Create a user-friendly layout for displaying weather information.\n- Step 2: Implement FastAPI endpoints to fetch and display weather data.", scope: None, urls: None, frontend_code: None, backend_code: None, api_schema: None }
```

Finally, ManagerGPT aggregates the outcomes from all GPTs, consolidating the generated tasks into a coherent project plan. Here's the summary of the completed tasks:

```sh
[*] "ManagerGPT": Completed Tasks: Tasks { description: "- Utilizing FastAPI and Python, develop a user-friendly UI for displaying weather data.\n- Step 1: Use HTML, CSS, and JavaScript to design and develop web pages that showcase weather data effectively.\n- Step 2: Define routes in FastAPI for weather data retrieval and display, ensuring seamless integration between frontend and backend.", scope: None, urls: None, frontend_code: Some("from fastapi import FastAPI, Request\nfrom fastapi.responses import JSONResponse\nimport requests\n\napp = FastAPI()\n\n@app.get(\"/weather/{city_name}\")\nasync def get_weather(city_name: str, request: Request):\n    url = f\"http://api.openweathermap.org/data/2.5/weather?q={city_name}&appid=YOUR_API_KEY\"\n    response = requests.get(url)\n    if response.status_code == 200:\n        data = response.json()\n        return JSONResponse(content={\n            \"city_name\": data[\"name\"],\n            \"temperature\": data[\"main\"][\"temp\"] - 273.15,\n            \"humidity\": data[\"main\"][\"humidity\"],\n            \"wind_speed\": data[\"wind\"][\"speed\"]\n        })\n    else:\n        return JSONResponse(content={\"error\": \"Could not fetch weather data\"}, status_code=response.status_code)"), backend_code:  Some("from fastapi import FastAPI, Request, Form\nfrom fastapi.templating import Jinja2Templates\nfrom fastapi.responses import HTMLResponse\nimport requests\n\napp = FastAPI()\ntemplates = Jinja2Templates(directory=\"templates\")\n\n@app.get(\"/\", response_class=HTMLResponse)\nasync def index(request: Request):\n    return templates.TemplateResponse(\"index.html\", {\"request\": request})\n\n@app.post(\"/weather\", response_class=HTMLResponse)\nasync def weather(request: Request, city: str = Form(...)):\n    url = f\"https://api.openweathermap.org/data/2.5/weather?q={city}&appid=YOUR_API_KEY\"\n    response = requests.get(url)\n    data = response.json()\n    return templates.TemplateResponse(\"weather.html\", {\"request\": request, \"data\": data})"), api_schema: None }
```

### 2. 👷‍♀️ ArchitectGPT

<img src="https://github.com/kevin-rs/kevin/assets/62179149/91a4868a-093f-4c96-89fc-5447e6f904f1" align="left" alt="architect" width="64" />

ArchitectGPT is responsible for designing the overall structure and architecture of your application. ArchitectGPT will create the foundation upon which your app will be built.

#### How ArchitectGPT Works

Upon receiving instructions from ManagerGPT, ArchitectGPT will:

- Determine the technologies and frameworks needed to realize the project goals and arcitecture of the project using the [diagrams](https://github.com/mingrammer/diagrams) library.
- Design the data flow and communication between backend and frontend components to ensure seamless operation.

![weather_forecast_website_diagram](https://github.com/kevin-rs/kevin/assets/62179149/d94b852c-30d4-4699-a7b1-b8cc225d9bd3)

### 3. 🎨 DesignerGPT (Optional) Feature Flag: `img`

<img src="https://github.com/kevin-rs/kevin/assets/62179149/8f7ec0bd-392c-4263-a700-b3012d395479" align="left" alt="designer" width="64" />

DesignerGPT transforms ideas into visually stunning designs. Whether it's crafting sleek user interfaces or designing captivating user experiences, DesignerGPT brings your project to life with style.

#### How DesignerGPT Works

When tasked by ManagerGPT, DesignerGPT will:

- Create mockups and wireframes of the application's interface using imgget AI api, ensuring a user-friendly and intuitive design.
- Select colors, fonts, and layouts that align with your project's branding and aesthetic.
- TODO: Collaborate with other agents to integrate design elements seamlessly into the final product.

![DesignerGPT sample output](https://github.com/kevin-rs/kevin/assets/62179149/356cec29-e779-4f95-81d8-498ef07c1f3a)

### 4. ⚙️ BackendGPT

<img src="https://github.com/kevin-rs/kevin/assets/62179149/74819200-83d5-498a-9a43-658096145611" align="left" alt="backend" width="64" />

BackendGPT handles all things related to server-side logic and data processing. From database management to API integration, BackendGPT ensures that your application's backend is robust and efficient.

#### How BackendGPT Works

Upon receiving instructions from ManagerGPT, BackendGPT will:

- Develop the backend using FastAPI, implementing endpoints for retrieving weather data and handling user requests.
- Integrate external APIs or services to fetch real-time weather information.
- Ensure data security and integrity, implementing authentication and authorization mechanisms as needed.

![BackendGPT code](https://github.com/kevin-rs/kevin/assets/62179149/a9ec06e0-74be-4c0e-8e3a-751eb0389c90)

### 5. 🖥️ FrontendGPT

<img src="https://github.com/kevin-rs/kevin/assets/62179149/684da3ce-f36c-4e2e-a315-0a834ba39539" align="left" alt="frontend" width="64" />

FrontendGPT will craft engaging and interactive experiences for your application's users. With a keen eye for design and a knack for coding, FrontendGPT brings your designs to life in the browser.

#### How FrontendGPT Works

When prompted by ManagerGPT, FrontendGPT will:

- Develop the frontend interface using modern web technologies such as HTML, CSS, and JavaScript, complementing the backend's functionality.
- Implement responsive design principles to ensure a seamless experience across devices and screen sizes.
- TODO: Collaborate with DesignerGPT to translate design mockups into code, bringing the application's visual identity to fruition.

### 6. 💌 MailerGPT (Optional) Feature Flag: `mail`

<img src="https://github.com/kevin-rs/kevin/assets/62179149/fedaf721-20b4-43e6-bdb9-ef3f87430ec3" align="left" alt="mailer" width="64" />

MailerGPT streamlines your communication processes by automating the creation and distribution of emails.

#### How MailerGPT Works

MailerGPT operates by:

- Reading your emails and extracting relevant information based on the user input.
- Generating and sending personalized email content tailored to specific recipients or target audiences.

With Autogpt's team of specialized agents working together, your project is in capable hands. Simply provide a simple project goal, and let Autogpt handle the rest!

---

## 📚 Documentation

For detailed usage instructions and API documentation, refer to the [AutoGPT Documentation](https://docs.rs/autogpt).

---

## 🤝 Contributing

Contributions are welcome! See the [Contribution Guidelines](CONTRIBUTING.md) for more information on how to get started.

---

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---