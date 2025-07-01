# 💎 AutoGPT Gemini Example

This is an example demonstrating how to use AutoGPT with Gemini. It features a simple use case of an architect agent that generates `diagrams` code.

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
   cd autogpt/autogpt/examples/gemini
   ```

1. Set the following environment variables:

   ```sh
   export GEMINI_API_KEY=<your_gemini_api_key>

   # Optional: Set the Model, flash 2.0 is the default
   export GEMINI_MODEL=<your_gemini_model>
   ```

   Generate an api key from [Google AI Studio](https://aistudio.google.com/app/apikey).

1. Run the app:

   ```sh
   cargo run
   ```

   Notice the newly created `workspace/architect` directory.

1. Generate the diagram:

   ```sh
   ./workspace/architect/.venv/bin/python ./workspace/architect/diagram.py
   ```
