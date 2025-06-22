# 💎 AutoGPT Gemini Memory Example

This is an example demonstrating how to use AutoGPT with Gemini with a persistant storage using Pinecone. It features a simple use case of an architect agent that generates `diagrams` code.

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
   cd autogpt/examples/gemini-memory
   ```

1. Set the following environment variables:

   ```sh
   export GEMINI_API_KEY=<your_gemini_api_key>
   # Optional: Set the Model, flash 2.0 is the default
   export GEMINI_MODEL=<your_gemini_model>

   export PINECONE_API_KEY=<Your_Pinecone_API_Key>
   export PINECONE_INDEX_URL=<Your_Pinecone_Index_URL>
   ```

   Generate an api key from [Google AI Studio](https://aistudio.google.com/app/apikey). Follow [our tutorial](../../PINECONE.md) for a guide on how to obtain Pinecone related keys.

1. Run the app:

   ```sh
   cargo run
   ```

   Executing this command will result in the following output:

   ```sh
   Execution completed with tools: [Diagram]
   Memory length: 4
   First memory role: user
   Second memory role: assistant
   Agent status: Completed
   ```
