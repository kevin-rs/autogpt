# Feature Flags

AutoGPT's `Cargo.toml` uses feature flags to keep the binary lean. Only compile what you need. Features are additive: enable multiple with a comma-separated list.

## Full Feature Table

| Feature | Enables                                                                                   | Required Env Vars                        |
| ------- | ----------------------------------------------------------------------------------------- | ---------------------------------------- |
| `gem`   | Gemini LLM client (`gems` crate)                                                          | `GEMINI_API_KEY`                         |
| `oai`   | OpenAI client (`openai_dive` crate)                                                       | `OPENAI_API_KEY`                         |
| `cld`   | Anthropic Claude client                                                                   | `ANTHROPIC_API_KEY`                      |
| `xai`   | XAI Grok client (`x-ai` crate)                                                            | `XAI_API_KEY`                            |
| `co`    | Cohere client (`cohere-rust` crate)                                                       | `COHERE_API_KEY`                         |
| `hf`    | HuggingFace client (`api_huggingface` crate)                                              | `HF_API_KEY`                             |
| `mcp`   | Model Context Protocol (MCP) support                                                      | N/A                                      |
| `gpt`   | All built-in GPT agents (ManagerGPT, ArchitectGPT, BackendGPT, FrontendGPT, OptimizerGPT) | One LLM feature                          |
| `img`   | DesignerGPT image generation (`getimg` crate)                                             | `GETIMG_API_KEY`                         |
| `git`   | GitGPT auto-commit (`git2` crate)                                                         | Git repo in workspace                    |
| `mail`  | MailerGPT email integration (`nylas` crate)                                               | `NYLAS_*` vars                           |
| `mem`   | Pinecone long-term memory (`pinecone-sdk` crate)                                          | `PINECONE_API_KEY`, `PINECONE_INDEX_URL` |
| `net`   | IAC protocol networking (`iac-rs` crate)                                                  | `ORCHESTRATOR_ADDRESS`                   |
| `cli`   | Interactive shell, session mgmt, YAML/TOML parsing, terminal UI                           | One LLM feature                          |

## Common Combinations

**Minimal SDK, Gemini only:**

```toml
autogpt = { version = "0.4.5", features = ["gem", "gpt"] }
```

**Full CLI with Gemini:**

```toml
autogpt = { version = "0.4.5", features = ["gem", "gpt", "cli"] }
```

**SDK with memory:**

```toml
autogpt = { version = "0.4.5", features = ["gem", "gpt", "mem"] }
```

**Networked orchestrated mode:**

```toml
autogpt = { version = "0.4.5", features = ["gem", "gpt", "net", "cli"] }
```

**Everything:**

```toml
autogpt = { version = "0.4.5", features = ["gem", "gpt", "img", "git", "mail", "mem", "net", "cli"] }
```

Or equivalently from the command line:

```sh
cargo install autogpt --all-features
```

## Compile-Time Feature Guards

In the source code, all feature-gated APIs are guarded with `#[cfg(feature = "...")]`. If you call `DesignerGPT::new` without the `img` feature, compilation fails with a clear `#[cfg]` error rather than a runtime panic.

## The `AI_PROVIDER` Environment Variable

At runtime, `AI_PROVIDER` selects the active LLM client among all compiled-in providers:

```sh
export AI_PROVIDER=gemini     # uses gem feature
export AI_PROVIDER=openai     # uses oai feature
export AI_PROVIDER=anthropic  # uses cld feature
export AI_PROVIDER=xai        # uses xai feature
export AI_PROVIDER=cohere     # uses co feature
export AI_PROVIDER=huggingface # uses hf feature
```

If `AI_PROVIDER` is not set, AutoGPT defaults to `gemini`.

<div class="callout callout-tip">
<strong>💡 Tip</strong>
Enabling only the features you need significantly reduces compile time and final binary size. The release profile is already optimized for size (<code>opt-level = "z"</code>, LTO, symbol stripping).
</div>
