# collab-agent

Demonstrates attaching a `CollabPool` directly to an `AgentGPT` via the built-in `with_collab_pool` API introduced in autogpt v0.4.5.

## What it shows

- Using `CollabPool::from_providers()` to configure LLM providers.
- Attaching the pool to an `AgentGPT` with `.with_collab_pool(pool)`.
- Inspecting the pool via `.collab_pool()` or `.collab_pool_mut()`.

## Requirements

- At least **two** provider API keys configured so the pool has multiple members.

```sh
export GEMINI_API_KEY=your_gemini_key
export OPENAI_API_KEY=your_openai_key   # second provider
...
```

## Running

```sh
cargo run --example collab-agent
```

## Features required

The example is built with `features = ["col", "gem", "gpt"]`.
Add `xai`, `oai`, `cld`, `co`, or `hf` as needed for additional providers.

## TUI integration

To use collab mode in the full TUI binary:

```sh
cargo run --all-features --bin autogpt -- --collab
```
