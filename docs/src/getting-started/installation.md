# Installation

AutoGPT ships as two binaries in a single crate: `autogpt` (the agent runner) and `orchgpt` (the orchestrator). Choose your preferred installation method below.

## From crates.io

The fastest way to install both binaries:

```sh
cargo install autogpt --all-features
```

After installation, verify both binaries are available:

```sh
autogpt --version
orchgpt --version
```

<div class="callout callout-tip">
<strong>💡 Tip</strong>
Linux x86-64 is the recommended and best-tested platform. AutoGPT works on Windows and macOS but the interactive shell experience is optimized for Unix terminals.
</div>

## Pre-built Binaries

Download the latest pre-built executables directly from GitHub Releases without a Rust toolchain:

| Platform       | Download                                                                                    |
| -------------- | ------------------------------------------------------------------------------------------- |
| Linux x86-64   | [autogpt](https://github.com/wiseaidotdev/autogpt/releases/download/v0.4.5/autogpt)         |
| Windows x86-64 | [autogpt.exe](https://github.com/wiseaidotdev/autogpt/releases/download/v0.4.5/autogpt.exe) |

## Using Docker

Run AutoGPT inside an isolated container, no Rust installation required.

**AutoGPT agent container:**

```sh
docker run -it \
  -e GEMINI_API_KEY=<your_gemini_api_key> \
  -e PINECONE_API_KEY=<your_pinecone_api_key> \
  -e PINECONE_INDEX_URL=<your_pinecone_index_url> \
  --rm --name autogpt kevinrsdev/autogpt
```

**OrchGPT orchestrator container:**

```sh
docker run -it \
  -e GEMINI_API_KEY=<your_gemini_api_key> \
  -e PINECONE_API_KEY=<your_pinecone_api_key> \
  -e PINECONE_INDEX_URL=<your_pinecone_index_url> \
  --rm --name orchgpt kevinrsdev/orchgpt
```

## Build from Source

Building from source gives you access to all development features and the latest unreleased code.

**1. Clone the repository:**

```sh
git clone https://github.com/wiseaidotdev/autogpt.git
cd autogpt/autogpt
```

**2. Build and run the agent CLI:**

```sh
cargo run --all-features --bin autogpt
```

**3. Build and run the orchestrator:**

```sh
cargo run --all-features --bin orchgpt
```

**4. Or build an optimized release binary:**

```sh
cargo build --release --all-features
# Binaries land in: ./target/release/autogpt and ./target/release/orchgpt
```

## SDK Installation

To embed AutoGPT in your own Rust project, add it to your `Cargo.toml`:

```toml
[dependencies]
autogpt = { version = "0.4.5", features = ["gem", "gpt"] }
tokio = { version = "1", features = ["full"] }
```

Select only the [feature flags](../advanced/feature-flags.md) you need to minimize compile times and binary size.

<div class="callout callout-info">
<strong>ℹ️ Note</strong>
AutoGPT requires Rust 1.89 or newer. Run <code>rustup update stable</code> to ensure you have a compatible toolchain.
</div>
