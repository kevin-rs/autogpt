# 💎 AutoGPT IAC Protocol Gemini Example

This example demonstrates how to use the IAC Protocol in AutoGPT with Gemini. It showcases a simple scenario where two agents, designer and frontend, communicate with an IAC server to delegate and coordinate tasks effectively.

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
   cd autogpt/examples/gemini-iac-client-server
   ```

1. Set the following environment variables:

   ```sh
   export GEMINI_API_KEY=<your_gemini_api_key>

   # Optional: Set the Model, flash 2.0 is the default
   export GEMINI_MODEL=<your_gemini_model>
   ```

   Generate an api key from [Google AI Studio](https://aistudio.google.com/app/apikey).

1. Start the server:

   ```sh
   cargo run -p server
   ```

1. In a new terminal, run the designer:

   ```sh
   cargo run -p designer
   ```

1. In another terminal, run the frontend:

   ```sh
   cargo run -p frontend
   ```
