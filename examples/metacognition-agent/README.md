# metacognition-agent

Demonstrates the `MetacognitionEngine` embedded in `AgentGPT` when built with the `mta` feature.

## What it shows

- Recording task outcomes with `agent.record_task_outcome(task, outcome, retries)`.
- Reading per-task insights and auto-generated strategy adjustments.
- Injecting the full metacognition context into an LLM prompt via `agent.metacognition_context()`.
- Querying `agent.should_adjust_strategy()` and `agent.consecutive_failures()`.

## Requirements

```sh
export GEMINI_API_KEY=your_gemini_key
```

## Running

```sh
cargo run
```

## Features required

The example is built with `features = ["mta", "gem", "gpt"]`.

## TUI integration

Enable metacognition in the TUI settings panel (tab 4 → Metacognition toggle) or set it in `~/.autogpt/settings.json`:

```json
{
  "metacognition": true
}
```
