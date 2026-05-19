// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::common::utils::{AgentIntent, strip_code_blocks};
use crate::prompts::generic::INTENT_DETECTION_PROMPT;
use anyhow::Result;
use serde::Deserialize;

/// Parsed JSON response from the LLM for intent detection.
///
/// The LLM is prompted to output a JSON object with an `intent` discriminant plus
/// optional `tool` and `args` fields for tool-call intents.
#[derive(Debug, Deserialize)]
struct IntentResponse {
    intent: String,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    args: Option<serde_json::Value>,
}

/// Type alias for the asynchronous LLM generator closure passed to `classify_intent`.
pub type GenerateFn =
    dyn FnMut(&str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + Send>>;

/// Classifies the user's message into one of three execution modes.
///
/// Builds the `INTENT_DETECTION_PROMPT` with the provided `workspace_snapshot` and
/// `mcp_tools` context, calls the `generate_fn` LLM closure, and parses the JSON
/// response into an `AgentIntent`. Any parsing failure falls back to `TaskPlan` so
/// the pipeline never blocks on a malformed response.
pub async fn classify_intent(
    prompt: &str,
    workspace_snapshot: &str,
    mcp_tools: &str,
    generate_fn: &mut GenerateFn,
) -> AgentIntent {
    let full = INTENT_DETECTION_PROMPT
        .replace("{USER_PROMPT}", prompt)
        .replace("{WORKSPACE}", workspace_snapshot)
        .replace("{MCP_TOOLS}", mcp_tools);

    let raw = match generate_fn(&full).await {
        Ok(r) => r,
        Err(_) => return AgentIntent::TaskPlan,
    };

    let clean = strip_code_blocks(&raw);

    let parsed: IntentResponse = match serde_json::from_str(clean.trim()) {
        Ok(p) => p,
        Err(_) => return AgentIntent::TaskPlan,
    };

    match parsed.intent.as_str() {
        "direct_answer" => AgentIntent::DirectAnswer,
        "tool_call" => AgentIntent::ToolCall {
            tool: parsed.tool.unwrap_or_else(|| "list_dir".to_string()),
            args: parsed.args.unwrap_or(serde_json::Value::Null),
        },
        _ => AgentIntent::TaskPlan,
    }
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
