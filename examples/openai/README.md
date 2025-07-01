# ֎ AutoGPT OpenAI Example

This is an example demonstrating how to use AutoGPT with OpenAI API. It features a simple use case of an architect agent that generates `diagrams` code.

## 🛠️ Pre-requisites:

### 🐧 **Linux Users**

1. **Install [`rustup`](https://www.rust-lang.org/tools/install)**:

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

### 🪟 **Windows Users**

1. **Download and install `rustup`**: Follow the installation instructions [here](https://forge.rust-lang.org/infra/other-installation-methods.html).

## 🚀 Building and Running

1. Fork/Clone the GitHub repository.

   ```sh
   git clone https://github.com/kevin-rs/autogpt
   ```

1. Navigate to the application directory.

   ```sh
   cd autogpt/autogpt/examples/openai
   ```

1. Set the following environment variables:

   ```sh
   export AI_PROVIDER=openai

   export OPENAI_API_KEY=<your_openai_api_key>
   ```

   Generate an api key from [OpenAI platform](https://platform.openai.com/docs/overview).

1. Run the app:

   ```sh
   cargo run
   ```

   Notice the newly created `workspace/architect` directory.

1. Generate the diagram:

   ```sh
   ./workspace/architect/.venv/bin/python ./workspace/architect/diagram.py
   ```
