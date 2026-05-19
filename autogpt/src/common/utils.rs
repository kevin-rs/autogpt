// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Utils module.
//!
//! This module provides various utility functions and common structures that can be used across different parts of the project.
//!
//! ## Structures
//!
//! - `Message`: Represents a message between agents.
//! - `Status`: Represents the status of an agent.
//! - `Route`: Represents a route object.
//! - `Scope`: Represents the scope of a project.
//! - `Task`: Represents a fact tasks.
//!
//! ## Functions
//!
//! - `extract_json_string`: Extracts a JSON string from the provided text.
//! - `extract_array`: Extracts an array from the provided text.
//! - `similarity`: Calculates the similarity between two strings using Levenshtein distance.
//! - `strip_code_blocks`: Strips code blocks from the provided text.
//!
//! # Examples
//!
//! ```
//! use autogpt::common::utils::{Message, Status, Route, Scope, Task, extract_json_string, extract_array, similarity, strip_code_blocks};
//!
//! let message = Message {
//!     role: "Sender".into(),
//!     content: "Hello, how are you?".into(),
//! };
//!
//! let status = Status::Idle;
//!
//! let route = Route {
//!     dynamic: "Yes".into(),
//!     method: "GET".into(),
//!     body: serde_json::json!({}),
//!     response: serde_json::json!({}),
//!     path: "/api".into(),
//! };
//!
//! let scope = Scope {
//!     crud: true,
//!     auth: true,
//!     external: false,
//! };
//!
//! let task = Task {
//!     description: "This is a task description.".into(),
//!     scope: Some(scope),
//!     urls: Some(vec!["https://wiseai.dev".into()]),
//!     frontend_code: None,
//!     backend_code: None,
//!     api_schema: None,
//! };
//!
//! let json_string = "{ \"crud\": true }";
//! let extracted_json = extract_json_string(json_string);
//!
//! let text = "[\"item1\", \"item2\"]";
//! let extracted_array = extract_array(text);
//!
//! let similarity = similarity("hello", "helo");
//!
//! let code_with_blocks = "```\nSome code here\n```";
//! let stripped_code = strip_code_blocks(code_with_blocks);
//! ```

#[cfg(feature = "cli")]
pub use crate::agents::agent::AgentGPT;
#[allow(unused_imports)]
pub use crate::traits::agent::Agent;
use chrono::prelude::*;
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::env::var;
#[allow(unused_imports)]
use std::sync::Arc;
#[cfg(feature = "cli")]
use {
    colored::Colorize,
    crates_io_api::AsyncClient,
    indicatif::{ProgressBar, ProgressStyle},
    semver::Version,
    std::{
        error,
        io::{self, Read, Write},
        process::{Child, Command, Stdio},
        time::Duration,
    },
    tracing::{Event, Subscriber, error, field, info, warn},
    tracing_appender::rolling,
    tracing_subscriber::Layer,
    tracing_subscriber::Registry,
    tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields},
    tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt,
    tracing_subscriber::registry::LookupSpan,
    tracing_subscriber::{filter, fmt},
    webbrowser::{Browser, BrowserOptions, open_browser_with_options},
};

#[cfg(feature = "xai")]
use x_ai::{chat_compl::Message as XaiMessage, client::XaiClient, traits::ClientConfig};

#[cfg(feature = "gem")]
pub use gems::{
    Client as GeminiClient,
    chat::ChatBuilder,
    messages::{Content, Message as GemMessage},
    models::Model as GeminiModel,
    traits::CTrait,
};

#[cfg(feature = "oai")]
pub use openai_dive::v1::{
    api::Client as OpenAIClient,
    models::Gpt4Model,
    resources::chat::{ChatMessage, ChatMessageContent},
};

#[cfg(feature = "cld")]
pub use anthropic_ai_sdk::{
    client::AnthropicClient,
    types::message::{Message as AnthMessage, MessageError, Role},
};

#[cfg(feature = "co")]
use {cohere_rust::Cohere, std::ops};

#[cfg(feature = "hf")]
use api_huggingface::{
    Client as HfApiClient, environment::HuggingFaceEnvironmentImpl, secret::Secret,
};

/// Maps a Hugging Face model ID string to a canonical model ID supported by
/// the HF Router API (`router.huggingface.co/v1/chat/completions`).
///
/// # Arguments
///
/// * `model` - Model ID string or short alias.
///
/// # Returns
///
/// (`&'static str`): The canonical model ID accepted by the router, defaulting
/// to `meta-llama/Llama-3.3-70B-Instruct` for unknown strings.
#[cfg(feature = "hf")]
pub fn hf_model_from_str(model: &str) -> &'static str {
    use api_huggingface::components::models::Models;
    match model {
        "meta-llama/Llama-3.3-70B-Instruct" | "llama-3.3" | "llama" | "llama3" => {
            Models::llama_3_3_70b_instruct()
        }
        "moonshotai/Kimi-K2-Instruct-0905" | "kimi" | "kimi-k2" => Models::kimi_k2_instruct(),
        "mistralai/Mistral-7B-Instruct-v0.3" | "mistral" | "mistral-7b" => {
            Models::mistral_7b_instruct()
        }
        "codellama/CodeLlama-7b-Instruct-hf" | "codellama" => Models::code_llama_7b_instruct(),
        "gpt2" => Models::gpt2(),
        s if s.contains('/') => Models::llama_3_3_70b_instruct(),
        _ => Models::llama_3_3_70b_instruct(),
    }
}

/// A thin wrapper around `api_huggingface::Client` that carries the selected model identifier.
#[cfg(feature = "hf")]
#[derive(Clone)]
pub struct HuggingFaceClient {
    /// The model ID string (e.g. `"meta-llama/Llama-3.3-70B-Instruct"`).
    pub model: String,
    /// Shared, stateless Hugging Face API client.
    pub client: Arc<HfApiClient<HuggingFaceEnvironmentImpl>>,
}

#[cfg(feature = "hf")]
impl std::fmt::Debug for HuggingFaceClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HuggingFaceClient(model={})", self.model)
    }
}

#[cfg(feature = "hf")]
impl HuggingFaceClient {
    /// Constructs a `HuggingFaceClient` from an explicit API key and model ID string.
    ///
    /// # Arguments
    ///
    /// * `api_key` - A valid Hugging Face API token (`hf_...`).
    /// * `model`   - Model ID in `org/model` format or a short alias recognised by `hf_model_from_str`.
    ///
    /// # Returns
    ///
    /// (`HuggingFaceClient`): A new client ready to call the HF Inference API.
    ///
    /// # Panics
    ///
    /// Panics if the underlying `api_huggingface` environment or client cannot be built.
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        let api_key = api_key.into();
        let model = model.into();
        let secret = Secret::new(api_key);
        let env = HuggingFaceEnvironmentImpl::build(secret, None)
            .expect("Failed to build HuggingFace environment");
        let client = HfApiClient::build(env).expect("Failed to build HuggingFace client");
        Self {
            model,
            client: Arc::new(client),
        }
    }
}

#[cfg(feature = "co")]
#[derive(Clone)]
pub struct CohereClient(pub Arc<Cohere>);

#[cfg(feature = "co")]
impl std::fmt::Debug for CohereClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CohereClient")
    }
}

#[cfg(feature = "co")]
impl ops::Deref for CohereClient {
    type Target = Cohere;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Enum representing supported AI clients.
#[derive(Debug, Clone)]
pub enum ClientType {
    /// OpenAI client.
    #[cfg(feature = "oai")]
    OpenAI(OpenAIClient),

    /// Google Gemini client.
    #[cfg(feature = "gem")]
    Gemini(GeminiClient),

    /// Anthropic Claude client.
    #[cfg(feature = "cld")]
    Anthropic(AnthropicClient),

    /// xAI Grok client.
    #[cfg(feature = "xai")]
    Xai(XaiClient),

    /// Cohere client.
    #[cfg(feature = "co")]
    Cohere(CohereClient),

    /// Hugging Face Inference API client.
    #[cfg(feature = "hf")]
    HuggingFace(HuggingFaceClient),

    /// No client configured or features disabled.
    None,
}

impl Default for ClientType {
    fn default() -> Self {
        ClientType::from_env()
    }
}

impl ClientType {
    /// Constructs the appropriate client from environment variables.
    ///
    /// Reads `AI_PROVIDER` to select the backend, then reads the provider-specific
    /// environment variables (`HF_API_KEY`, `HF_MODEL`, `GEMINI_API_KEY`, etc.).
    ///
    /// # Panics
    ///
    /// Panics if `AI_PROVIDER` is set to a value that requires a feature flag that is
    /// not compiled in, or if a required API key env var is missing.
    pub fn from_env() -> Self {
        let provider = var("AI_PROVIDER").unwrap_or_else(|_| "gemini".to_string());
        #[allow(unused)]
        let _p = &provider;

        #[cfg(feature = "hf")]
        if provider == "huggingface" {
            let api_key = var("HF_API_KEY").expect("Missing HF_API_KEY");
            let model =
                var("HF_MODEL").unwrap_or_else(|_| "meta-llama/Llama-3.3-70B-Instruct".to_string());
            return ClientType::HuggingFace(HuggingFaceClient::new(api_key, model));
        }

        #[cfg(feature = "co")]
        if provider == "cohere" {
            let client = CohereClient(Arc::new(Cohere::default()));
            return ClientType::Cohere(client);
        }

        #[cfg(feature = "oai")]
        if provider == "openai" {
            let openai_client = OpenAIClient::new_from_env();
            return ClientType::OpenAI(openai_client);
        }

        #[cfg(feature = "gem")]
        if provider == "gemini" {
            let model = var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3.0-flash".to_string());
            let api_key = var("GEMINI_API_KEY").unwrap_or_default();
            let gemini_client = GeminiClient::builder().model(&model).build().unwrap();
            gemini_client.set_api_key(api_key);
            return ClientType::Gemini(gemini_client);
        }

        #[cfg(feature = "cld")]
        if provider == "anthropic" {
            let api_key = var("ANTHROPIC_API_KEY").expect("Missing ANTHROPIC_API_KEY");
            let client = AnthropicClient::new::<MessageError>(api_key, "2023-06-01")
                .expect("Failed to create Anthropic client");
            return ClientType::Anthropic(client);
        }

        #[cfg(feature = "xai")]
        if provider == "xai" {
            let api_key = var("XAI_API_KEY").expect("Missing XAI_API_KEY");
            let client = XaiClient::builder()
                .build()
                .expect("Failed to build XaiClient");
            client.set_api_key(api_key);
            return ClientType::Xai(client);
        }

        #[allow(unreachable_code)]
        {
            ClientType::None
        }
    }
}

/// Represents a message between agents.
#[derive(Eq, Debug, PartialEq, Default, Clone, Hash, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message.
    pub role: Cow<'static, str>,
    /// The content of the message.
    pub content: Cow<'static, str>,
}

/// Represents the status of an agent.
#[derive(Debug, PartialEq, Default, Clone)]
pub enum Status {
    /// Agent is in the discovery phase.
    #[default]
    Idle,
    /// Agent is actively working.
    Active,
    /// Agent is in the unit testing phase.
    InUnitTesting,
    /// Agent has finished its task.
    Completed,
}

/// Represents a route object.
#[derive(Eq, Debug, Serialize, Deserialize, Clone, PartialEq, Default, Hash)]
pub struct Route {
    /// Indicates if the route is dynamic.
    pub dynamic: Cow<'static, str>,
    /// The HTTP method of the route.
    pub method: Cow<'static, str>,
    /// The request body of the route.
    pub body: Value,
    /// The response of the route.
    pub response: Value,
    /// The route path.
    pub path: Cow<'static, str>,
}

/// Represents the scope of a project.
#[derive(Eq, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default, Hash)]
pub struct Scope {
    /// Indicates if CRUD operations are required.
    pub crud: bool,
    /// Indicates if user login and logout are required.
    pub auth: bool,
    /// Indicates if external URLs are required.
    pub external: bool,
}

/// Represents a fact tasks.
#[derive(Eq, Debug, Serialize, Deserialize, Clone, PartialEq, Default, Hash)]
pub struct Task {
    /// The description of the project.
    pub description: Cow<'static, str>,
    /// The scope of the project.
    pub scope: Option<Scope>,
    /// External URLs required by the project.
    pub urls: Option<Vec<Cow<'static, str>>>,
    /// Frontend code of the project.
    pub frontend_code: Option<Cow<'static, str>>,
    /// Backend code of the project.
    pub backend_code: Option<Cow<'static, str>>,
    /// Schema of API endpoints.
    pub api_schema: Option<Vec<Route>>,
}

impl Task {
    pub fn from_payload(payload: &str) -> Self {
        Task {
            description: payload.to_string().into(),
            scope: None,
            urls: None,
            frontend_code: None,
            backend_code: None,
            api_schema: None,
        }
    }
}

pub fn extract_json_string(text: &str) -> Option<String> {
    if let Some(start_index) = text.find("{\n  \"crud\"") {
        let mut end_index = start_index + 1;
        let mut open_braces_count = 1;

        for (i, c) in text[start_index + 1..].char_indices() {
            match c {
                '{' => open_braces_count += 1,
                '}' => {
                    open_braces_count -= 1;
                    if open_braces_count == 0 {
                        end_index = start_index + i + 2;
                        break;
                    }
                }
                _ => {}
            }
        }

        return Some(text[start_index..end_index].to_string());
    }

    None
}

pub fn extract_array(text: &str) -> Option<String> {
    if text.starts_with('[') && text.ends_with(']') {
        Some(text.to_string())
    } else if let Some(start_index) = text.find("[\"") {
        let mut end_index = start_index + 1;
        let mut open_brackets_count = 1;

        for (i, c) in text[start_index + 1..].char_indices() {
            match c {
                '[' => open_brackets_count += 1,
                ']' => {
                    open_brackets_count -= 1;
                    if open_brackets_count == 0 {
                        end_index = start_index + i + 2;
                        break;
                    }
                }
                _ => {}
            }
        }

        Some(text[start_index..end_index].to_string())
    } else {
        None
    }
}

/// Extracts the first balanced `{...}` JSON object from arbitrary text.
///
/// Used as a fallback when LLM output wraps JSON in prose or markdown fences.
pub fn extract_json_object(text: &str) -> Option<String> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end >= start {
        Some(text[start..=end].to_string())
    } else {
        None
    }
}

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for (i, item) in matrix.iter_mut().enumerate().take(len1 + 1) {
        item[0] = i;
    }

    for (j, item) in matrix[0].iter_mut().enumerate() {
        *item = j;
    }

    for (i, char1) in s1.chars().enumerate() {
        for (j, char2) in s2.chars().enumerate() {
            let cost = if char1 == char2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[len1][len2]
}

pub fn similarity(s1: &str, s2: &str) -> f64 {
    let distance = levenshtein_distance(s1, s2) as f64;
    let max_length = s1.chars().count().max(s2.chars().count()) as f64;
    1.0 - distance / max_length
}

pub fn strip_code_blocks(text: &str) -> String {
    if !text.contains("```") {
        return text.to_string();
    }

    let mut inside_block = false;
    let mut found_first = false;
    let mut result = Vec::new();

    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            if !found_first {
                found_first = true;
                inside_block = true;
                continue;
            } else if inside_block {
                break;
            }
        }

        if inside_block {
            result.push(line);
        }
    }

    result.join("\n")
}

pub fn is_yes(input: &str) -> bool {
    matches!(
        input.trim().to_lowercase().as_str(),
        "yes" | "y" | "si" | "sure" | "ok" | "okay"
    )
}

/// Runs a gpt project without generating new code.
///
/// # Arguments
///
/// * `language` - The programming language used ("rust", "python", "javascript").
/// * `path` - The working directory where the gpt project resides.
/// * `browse` - Whether to open the API docs in a browser.
///
/// # Returns
///
/// `Result<Option<Child>>` - The spawned gpt process (if successful), or an error.
#[cfg(feature = "cli")]
pub async fn run_code(
    language: &str,
    path: &str,
    browse: bool,
) -> Result<Option<Child>, Box<dyn error::Error + Send + Sync>> {
    if browse {
        let _ = open_browser_with_options(
            Browser::Default,
            "http://127.0.0.1:8000/docs",
            BrowserOptions::new().with_suppress_output(false),
        );
    }

    match language {
        "rust" => {
            let mut build_command = Command::new("cargo");
            build_command
                .arg("build")
                .arg("--release")
                .arg("--verbose")
                .current_dir(path);
            let build_output = build_command.output()?;

            if build_output.status.success() {
                let run_output = Command::new("timeout")
                    .arg("10s")
                    .arg("cargo")
                    .arg("run")
                    .arg("--release")
                    .arg("--verbose")
                    .current_dir(path)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?;
                Ok(Some(run_output))
            } else {
                Err("Rust build failed.".into())
            }
        }

        "python" => {
            let run_output = Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "timeout {} '.venv/bin/python' -m uvicorn main:app --host 0.0.0.0 --port 8000",
                    10
                ))
                .current_dir(path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to run the backend application");

            Ok(Some(run_output))
        }

        "javascript" => {
            let run_output = Command::new("timeout")
                .arg("10s")
                .arg("node")
                .arg("src/index.js")
                .current_dir(path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            Ok(Some(run_output))
        }

        _ => Err(format!("Unsupported language: {language}").into()),
    }
}
#[cfg(feature = "cli")]
pub struct NoLevelFormatter;

#[cfg(feature = "cli")]
impl<S, N> FormatEvent<S, N> for NoLevelFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: fmt::format::Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        struct MessageVisitor<'a, 'w>(&'a mut fmt::format::Writer<'w>);
        impl<'a, 'w> field::Visit for MessageVisitor<'a, 'w> {
            fn record_debug(&mut self, field: &field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "message" {
                    let _ = write!(self.0, "{:?}", value);
                }
            }
            fn record_str(&mut self, field: &field::Field, value: &str) {
                if field.name() == "message" {
                    let _ = write!(self.0, "{}", value);
                }
            }
        }
        let mut visitor = MessageVisitor(&mut writer);
        event.record(&mut visitor);
        writeln!(writer)
    }
}

#[cfg(feature = "cli")]
pub fn setup_logging(hide_console: bool) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        console::set_colors_enabled(true);
        console::set_colors_enabled_stderr(true);
    }
    colored::control::set_override(true);
    let file_appender = rolling::daily("logs", "autogpt_log");

    let console_filter = if hide_console {
        filter::LevelFilter::OFF
    } else {
        filter::LevelFilter::INFO
    };

    let console_layer = fmt::Layer::new()
        .compact()
        .without_time()
        .with_file(false)
        .with_line_number(false)
        .with_thread_ids(false)
        .with_target(false)
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .event_format(NoLevelFormatter)
        .with_filter(console_filter);

    let file_layer = fmt::Layer::new()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(true)
        .with_writer(file_appender)
        .with_filter(filter::LevelFilter::DEBUG);

    let subscriber = Registry::default().with(console_layer).with(file_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

#[cfg(feature = "cli")]
pub async fn ask_to_run_command(
    agent: AgentGPT,
    language: &str,
    workspace: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !agent.memory().is_empty() {
        warn!(
            "{}",
            "[*] \"AGI\": Maybe it's time to run the application? (yes/no)"
                .bright_yellow()
                .bold()
        );

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if is_yes(&input) {
            info!(
                "{}",
                "[*] \"AGI\": 🫡 Roger! Running the application..."
                    .green()
                    .bold()
            );

            let result = run_code(language, workspace, true).await;

            match result {
                Ok(Some(mut child)) => {
                    let _build_stdout =
                        child.stdout.take().expect("Failed to capture build stdout");
                    let mut build_stderr =
                        child.stderr.take().expect("Failed to capture build stderr");

                    let mut stderr_output = String::new();
                    build_stderr.read_to_string(&mut stderr_output)?;

                    if !stderr_output.trim().is_empty() {
                        error!(
                            "{}",
                            "[*] \"AGI\": Too many bugs found. Consider debugging..."
                                .bright_red()
                                .bold()
                        );
                    } else {
                        info!(
                            "{}",
                            "[*] \"AGI\": Application built successful..."
                                .bright_white()
                                .bold()
                        );
                    }
                }
                Err(e) => {
                    error!("{}", format!("[*] \"AGI\": Error: {e}").bright_red().bold());
                }
                _ => {}
            }
        }
    }

    Ok(())
}

/// Enum representing supported GPT models.
#[derive(Debug, PartialEq, Clone)]
pub enum Model {
    /// Cohere model.
    #[cfg(feature = "co")]
    Cohere(String),

    /// OpenAI model.
    #[cfg(feature = "oai")]
    OpenAI(String),

    /// Google Gemini model.
    #[cfg(feature = "gem")]
    Gemini(GeminiModel),

    /// Anthropic Claude model.
    #[cfg(feature = "cld")]
    Claude(String),

    /// xAI Grok model.
    #[cfg(feature = "xai")]
    Xai(String),

    /// Hugging Face model identifier in `org/model` format.
    #[cfg(feature = "hf")]
    HuggingFace(String),
}

impl Default for Model {
    fn default() -> Self {
        #[cfg(feature = "gem")]
        {
            Model::Gemini(GeminiModel::Flash3Preview)
        }

        #[cfg(all(not(feature = "gem"), feature = "co"))]
        {
            Model::Cohere("command-a-03-2025".to_string())
        }

        #[cfg(all(not(any(feature = "gem", feature = "co")), feature = "oai"))]
        {
            Model::OpenAI("gpt-5".to_string())
        }

        #[cfg(all(
            not(any(feature = "gem", feature = "co", feature = "oai")),
            feature = "cld"
        ))]
        {
            Model::Claude("claude-opus-4-6".to_string())
        }

        #[cfg(all(
            not(any(feature = "gem", feature = "co", feature = "oai", feature = "cld")),
            feature = "xai"
        ))]
        {
            Model::Xai("grok-4".to_string())
        }

        #[cfg(all(
            not(any(
                feature = "gem",
                feature = "co",
                feature = "oai",
                feature = "cld",
                feature = "xai"
            )),
            feature = "hf"
        ))]
        {
            Model::HuggingFace("mistralai/Mistral-7B-Instruct-v0.3".to_string())
        }

        #[cfg(not(any(
            feature = "co",
            feature = "oai",
            feature = "gem",
            feature = "cld",
            feature = "xai",
            feature = "hf"
        )))]
        {
            panic!(
                "At least one of the features `hf`, `co`, `oai`, `gem`, `cld`, or `xai` must be enabled for Model::default()"
            );
        }
    }
}

#[derive(Debug, Clone)]
pub enum ProviderMessage {
    /// Cohere message type (plain text prompt).
    #[cfg(feature = "co")]
    Cohere(String),

    /// OpenAI message type.
    #[cfg(feature = "oai")]
    OpenAI(ChatMessage),

    /// Google Gemini message type.
    #[cfg(feature = "gem")]
    Gemini(GemMessage),

    /// Anthropic Claude message type.
    #[cfg(feature = "cld")]
    Claude(AnthMessage),

    /// xAI Grok message type.
    #[cfg(feature = "xai")]
    Xai(XaiMessage),

    /// Hugging Face plain-text prompt.
    #[cfg(feature = "hf")]
    HuggingFace(String),
}

impl Default for ProviderMessage {
    fn default() -> Self {
        #[cfg(feature = "co")]
        {
            ProviderMessage::Cohere("Hello".to_string())
        }

        #[cfg(all(not(feature = "co"), feature = "oai"))]
        {
            ProviderMessage::OpenAI(ChatMessage::User {
                content: ChatMessageContent::Text("Hello".into()),
                name: None,
            })
        }

        #[cfg(all(not(any(feature = "co", feature = "oai")), feature = "cld"))]
        {
            ProviderMessage::Claude(AnthMessage::new_text(Role::User, "Hello"))
        }

        #[cfg(all(
            not(any(feature = "co", feature = "oai", feature = "cld")),
            feature = "gem"
        ))]
        {
            ProviderMessage::Gemini(GemMessage::User {
                content: Content::Text("Hello".into()),
                name: None,
            })
        }

        #[cfg(all(
            not(any(feature = "co", feature = "oai", feature = "cld", feature = "gem")),
            feature = "xai"
        ))]
        {
            ProviderMessage::Xai(XaiMessage::text("user", "Hello"))
        }

        #[cfg(all(
            not(any(
                feature = "co",
                feature = "oai",
                feature = "cld",
                feature = "gem",
                feature = "xai"
            )),
            feature = "hf"
        ))]
        {
            ProviderMessage::HuggingFace("Hello".to_string())
        }

        #[cfg(not(any(
            feature = "co",
            feature = "oai",
            feature = "gem",
            feature = "cld",
            feature = "xai",
            feature = "hf"
        )))]
        {
            panic!(
                "At least one of the features `hf`, `co`, `oai`, `gem`, `cld`, or `xai` must be enabled for ProviderMessage::default()"
            );
        }
    }
}

impl ProviderMessage {
    /// Constructs a `ProviderMessage` from a plain-text string using the first enabled provider.
    pub fn from_text(_text: impl Into<String>) -> Self {
        #[cfg(feature = "co")]
        {
            ProviderMessage::Cohere(_text.into())
        }

        #[cfg(all(not(feature = "co"), feature = "oai"))]
        {
            ProviderMessage::OpenAI(ChatMessage::User {
                content: ChatMessageContent::Text(_text.into()),
                name: None,
            })
        }

        #[cfg(all(not(any(feature = "co", feature = "oai")), feature = "cld"))]
        {
            ProviderMessage::Claude(AnthMessage::new_text(Role::User, _text.into()))
        }

        #[cfg(all(
            not(any(feature = "co", feature = "oai", feature = "cld")),
            feature = "gem"
        ))]
        {
            ProviderMessage::Gemini(GemMessage::User {
                content: Content::Text(_text.into()),
                name: None,
            })
        }

        #[cfg(all(
            not(any(feature = "co", feature = "oai", feature = "cld", feature = "gem")),
            feature = "xai"
        ))]
        {
            ProviderMessage::Xai(XaiMessage {
                role: "user".to_string(),
                content: x_ai::chat_compl::MessageContent::Text(_text.into()),
                reasoning_content: None,
                refusal: None,
                tool_calls: None,
            })
        }

        #[cfg(all(
            not(any(
                feature = "co",
                feature = "oai",
                feature = "cld",
                feature = "gem",
                feature = "xai"
            )),
            feature = "hf"
        ))]
        {
            ProviderMessage::HuggingFace(_text.into())
        }

        #[cfg(not(any(
            feature = "co",
            feature = "oai",
            feature = "gem",
            feature = "cld",
            feature = "xai",
            feature = "hf"
        )))]
        {
            panic!(
                "At least one of the features `hf`, `co`, `oai`, `gem`, `cld`, or `xai` must be enabled for ProviderMessage::from_text()"
            );
        }
    }
}

/// Represents the standardized or custom name of a tool the agent can use.
#[derive(Eq, Debug, PartialEq, Default, Clone, Hash)]
pub enum ToolName {
    /// Web & Information Retrieval
    #[default]
    Search,
    Browser,
    News,
    Wiki,

    /// Data & Computation
    Calc,
    Math,
    Convert,
    Format,
    Sheet,

    /// Programming & Code Execution
    Exec,
    Code,
    Regex,
    Box,

    /// File & Document Handling
    Read,
    Write,
    Pdf,
    Summarize,

    /// Message & Scheduling
    Email,
    Sms,
    Calendar,
    Notes,

    /// Natural Language Processing
    Translate,
    Sentiment,
    Entities,
    TLDR,
    Classify,

    /// Media Understanding & Generation
    ImgGen,
    ImgScan,
    Transcribe,
    VidSum,

    /// Memory & Persistence
    VSearch,
    Memory,
    KB,
    Pad,

    /// System & External Integration
    Shell,
    Git,
    DB,
    API,

    /// Autonomy & Agentic Reasoning
    Plan,
    Spawn,
    Judge,
    Loop,

    /// Simulation & Modeling
    Diagram,
    Sim,
    Finance,

    /// Optimization or code performance improvement
    Optimize,

    /// UI development
    Frontend,

    /// Backend and server logic development
    Backend,

    /// Custom or plugin-based tool with a user-defined name.
    Plugin(String),
}

/// Represents a utility or function available to the agent, identified by a `ToolName`.
#[derive(Derivative)]
#[derivative(Eq, Debug, PartialEq, Default, Clone, Hash)]
pub struct Tool {
    /// The name/type of the tool.
    pub name: ToolName,
    /// A brief description of the tool's function.
    pub description: Cow<'static, str>,
    /// A function pointer to invoke the tool with a string input.
    #[derivative(Default(value = "noop_tool"), PartialEq = "ignore", Hash = "ignore")]
    pub invoke: fn(&str) -> String,
}

/// Represents a simple structured knowledge base for storing facts.
#[derive(Derivative)]
#[derivative(Eq, Debug, PartialEq, Default, Clone, Hash)]
pub struct Knowledge {
    /// A map of facts where the key is the identifier and the value is the explanation.
    #[derivative(Hash = "ignore")]
    pub facts: HashMap<Cow<'static, str>, Cow<'static, str>>,
}

/// Responsible for maintaining a current plan consisting of multiple goals.
#[derive(Eq, Debug, PartialEq, Default, Clone, Hash)]
pub struct Planner {
    /// The current sequence of goals the agent is working on.
    pub current_plan: Vec<Goal>,
}

#[derive(Eq, Debug, PartialEq, Default, Clone, Hash)]
pub struct Goal {
    pub description: String,
    pub priority: u8,
    pub completed: bool,
}

/// Represents the personality and behavioral traits of the agent.
#[derive(Eq, Debug, PartialEq, Default, Clone, Hash)]
pub struct Persona {
    /// The name or label of the persona.
    pub name: Cow<'static, str>,
    /// Traits describing the agent's personality.
    pub traits: Vec<Cow<'static, str>>,
    /// Optional behavior script (e.g., a DSL or JSON configuration).
    pub behavior_script: Option<Cow<'static, str>>,
}

/// A module for evaluating and reflecting on the agent's actions and thoughts.
#[derive(Derivative)]
#[derivative(Eq, Debug, PartialEq, Default, Clone, Hash)]
pub struct Reflection {
    /// A log of recent activities or messages.
    pub recent_logs: Vec<Cow<'static, str>>,
    /// A function for evaluating the agent's internal state.
    #[derivative(
        Default(value = "default_eval_fn"),
        PartialEq = "ignore",
        Hash = "ignore"
    )]
    pub evaluation_fn: fn(&dyn Agent) -> Cow<'static, str>,
}

/// A scheduler for managing the agent's future tasks.
#[derive(Eq, Debug, PartialEq, Default, Clone, Hash)]
pub struct TaskScheduler {
    /// A list of scheduled tasks with specific times.
    pub scheduled_tasks: Vec<ScheduledTask>,
}

/// Represents a task that is scheduled to occur at a certain time.
#[derive(Eq, Debug, PartialEq, Default, Clone, Hash)]
pub struct ScheduledTask {
    /// The scheduled time for the task.
    pub time: DateTime<Utc>,
    /// The goal associated with the task.
    pub task: Task,
}

/// Represents a sensor or input modality that the agent can use.
#[derive(Eq, Debug, PartialEq, Default, Clone, Hash)]
pub enum Sensor {
    /// Watches a file for changes.
    FileWatcher(Cow<'static, str>),
    /// Listens to an API endpoint for updates.
    ApiListener(Cow<'static, str>),
    /// Captures audio input.
    #[default]
    AudioInput,
    /// Uses a camera input stream.
    Camera,
    /// A custom sensor defined by a string identifier.
    Custom(Cow<'static, str>),
}

/// Enumerates possible capabilities the agent can possess.
#[derive(Eq, Debug, PartialEq, Default, Clone, Hash, Serialize, Deserialize)]
pub enum Capability {
    /// Can generate code from prompts.
    #[default]
    CodeGen,
    /// Can generate UI components.
    UIDesign,
    /// Can perform live web searches.
    WebSearch,
    /// Can access SQL databases.
    SQLAccess,
    /// Can control robotic hardware.
    RobotControl,
    /// Can interact with APIs.
    ApiIntegration,
    /// Can convert text to speech.
    TextToSpeech,
}

/// Manages recent messages and topics of focus for context maintenance.
#[derive(Eq, Debug, PartialEq, Default, Clone, Hash)]
pub struct ContextManager {
    /// Recent messages exchanged by the agent.
    pub recent_messages: Vec<Message>,
    /// Topics currently prioritized or focused on.
    pub focus_topics: Vec<Cow<'static, str>>,
}

/// Represents the primary mission or intent of an agent.
#[derive(Eq, Debug, PartialEq, Default, Clone, Hash)]
pub enum Behavior {
    /// Explore a given environment or dataset.
    #[default]
    Explore,
    /// Defend a target or state.
    Defend,
    /// Perform research and gather information.
    Research,
    /// Assist other agents or users.
    Assist,
    /// A custom behavior specified by the user.
    Custom(Cow<'static, str>),
}

/// Represents the spatial or logical location of an agent.
#[derive(Eq, Debug, PartialEq, Default, Clone, Hash)]
pub enum PersonaType {
    /// Frontline persona (e.g., high activity).
    #[default]
    Frontline,
    /// Support persona.
    Support,
    /// Reconnaissance or scout role.
    Recon,
    /// Strategic or command-level persona.
    Strategic,
    /// A custom-defined persona.
    Custom(Cow<'static, str>),
}

pub fn default_eval_fn(agent: &dyn Agent) -> Cow<'static, str> {
    if let Some(planner) = agent.planner() {
        let total = planner.current_plan.len();
        let completed = planner.current_plan.iter().filter(|g| g.completed).count();

        let mut summary = format!(
            "\n- Total Goals: {}\n- Completed: {}\n- In Progress: {}\n\nGoals Summary:\n",
            total,
            completed,
            total - completed
        );

        for (i, goal) in planner.current_plan.iter().enumerate() {
            let status = if goal.completed {
                "✅ Completed"
            } else {
                "⏳ In Progress"
            };
            summary.push_str(&format!("{}. {} [{}]\n", i + 1, goal.description, status));
        }

        Cow::Owned(summary)
    } else {
        Cow::Borrowed("No planner available for self-evaluation.")
    }
}

pub fn noop_tool(_: &str) -> String {
    "default tool output".to_string()
}

#[derive(Eq, Debug, PartialEq, Default, Clone, Hash)]
pub enum OutputKind {
    #[default]
    Text,
    UrlList,
    Scope,
}

#[derive(Eq, Debug, PartialEq, Clone, Hash)]
pub enum GenerationOutput {
    Text(String),
    UrlList(Vec<Cow<'static, str>>),
    Scope(Scope),
}

#[cfg(feature = "cli")]
pub fn spinner(label: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{prefix:.bold.dim} {spinner:.cyan} {msg}")
            .unwrap()
            .tick_chars("◑◒◐◓"),
    );
    pb.set_message(label.to_string());
    pb.enable_steady_tick(Duration::from_millis(120));
    pb
}

#[derive(Eq, Debug, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub enum AgentMessage {
    #[serde(rename = "task")]
    Task(Task),
    #[serde(rename = "status")]
    Status(String),
    #[serde(rename = "memory")]
    Memory(Vec<Message>),
    #[serde(rename = "capability_advert")]
    CapabilityAdvert {
        sender_id: String,
        capabilities: Vec<Capability>,
    },
    #[serde(rename = "custom")]
    Custom(String),
}

#[cfg(feature = "cli")]
#[allow(unused)]
pub async fn fetch_latest_version() -> Option<String> {
    let client = AsyncClient::new(
        "autogpt (github.com/wiseaidotdev/autogpt)",
        Duration::from_millis(1000),
    )
    .ok()?;

    let crate_data = client.get_crate("autogpt").await.ok()?;
    Some(crate_data.crate_data.max_version)
}

#[cfg(feature = "cli")]
#[allow(unused)]
pub fn is_outdated(current: &str, latest: &str) -> bool {
    let current = Version::parse(current).ok();
    let latest = Version::parse(latest).ok();
    current < latest
}

#[cfg(feature = "cli")]
#[allow(unused)]
pub fn prompt_for_update() {
    info!(
        "{}",
        "🚀 A new version of autogpt is available! Do you want to update? (y/N):"
            .bright_yellow()
            .bold()
    );

    print!("> ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        if input.trim().to_lowercase() == "y" {
            info!("{}", "🛠️  Updating autogpt...".bright_cyan().bold());

            let status = Command::new("cargo")
                .args(["install", "autogpt", "--force", "--all-features"])
                .status()
                .expect("❌ Failed to run cargo install");

            if status.success() {
                info!("{}", "✅ Successfully updated autogpt!".green().bold());
            } else {
                error!("{}", "❌ Failed to update autogpt.".red().bold());
            }
        } else {
            info!("{}", "❎ Skipping update.".dimmed());
        }
    }
}

/// The classified intent of a single user message, used to route execution
/// between direct LLM responses, tool invocations, and full task planning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum AgentIntent {
    /// A conversational or factual question the LLM can answer directly.
    DirectAnswer,
    /// Instructs the agent to invoke a named tool with the given JSON arguments.
    ToolCall {
        tool: String,
        #[serde(default)]
        args: serde_json::Value,
    },
    /// The prompt requires decomposing into sub-tasks and executing a full pipeline.
    TaskPlan,
}

/// The operational phase of the generic agent within a session lifecycle.
///
/// Transitions linearly from `Idle` through synthesis, planning, optional user
/// approval, per-task execution, per-task reflection, and finally `Complete`.
#[derive(Debug, Clone, PartialEq)]
pub enum PhaseState {
    Idle,
    Synthesizing,
    Planning,
    AwaitingApproval,
    Executing(usize),
    Reflecting,
    #[cfg(feature = "mta")]
    MetaCognizing,
    Complete,
}

/// A single structured action directive emitted by the LLM during task execution.
///
/// The LLM responds to execution prompts with a JSON array of `ActionRequest` values.
/// Each variant is one atomic filesystem, shell, git, web, or MCP operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ActionRequest {
    /// Create a directory (and any missing parent directories) at `path`.
    CreateDir { path: String },
    /// Create a new file at `path` with the given `content`.
    CreateFile { path: String, content: String },
    /// Overwrite an existing file at `path` with `content`.
    WriteFile { path: String, content: String },
    /// Read and return the full utf-8 content of the file at `path`.
    ReadFile { path: String },
    /// Replace the first occurrence of `old_text` with `new_text` inside `path`.
    PatchFile {
        path: String,
        old_text: String,
        new_text: String,
    },
    /// Append `content` to the end of the file at `path`.
    AppendFile { path: String, content: String },
    /// List all direct children of the directory at `path`.
    ListDir { path: String },
    /// Search for lines in `path` that contain the literal string `pattern`.
    FindInFile { path: String, pattern: String },
    /// Execute a shell command with `args`; optional `cwd` is relative to workspace root.
    RunCommand {
        cmd: String,
        args: Vec<String>,
        cwd: Option<String>,
    },
    /// Stage all files and create a git commit with `message`.
    GitCommit { message: String },
    /// Glob-match files under the workspace root and return matching relative paths.
    GlobFiles { pattern: String },
    /// Apply a list of `(old_text, new_text)` patches to `path` atomically.
    MultiPatch {
        path: String,
        patches: Vec<(String, String)>,
    },
    /// Perform a DuckDuckGo web search for `query` and return formatted results.
    WebSearch { query: String },
    /// Invoke an MCP tool on the named `server` with the given JSON `args`.
    McpCall {
        server: String,
        tool: String,
        #[serde(default)]
        args: serde_json::Value,
    },
}

/// The outcome of executing a single `ActionRequest`.
///
/// Aggregates the action type label, an optional affected path, captured
/// stdout and stderr strings, and a boolean success flag for reflection logic.
#[derive(Debug, Clone)]
pub struct ActionResult {
    pub action_type: String,
    pub path: Option<String>,
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

/// Structured inner-monologue produced by the LLM before executing each sub-task.
///
/// Parsed from a JSON response to the `REASONING_PROMPT`. When parsing fails
/// the agent substitutes a plain-text fallback so the pipeline never blocks.
#[derive(Debug, Deserialize, Default)]
pub struct ReasoningResult {
    pub thought: String,
    pub approach: String,
    #[serde(default)]
    pub risks: Vec<String>,
}

/// High-level verdict returned by the LLM after verifying a completed sub-task.
///
/// Drives the retry loop: `Success` breaks the loop, `Retry` runs corrective
/// actions up to the configured maximum, and `Skip` advances the pipeline.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReflectionOutcome {
    Success,
    Retry,
    Skip,
}

/// Full reflection response from the LLM including verdict and corrective actions.
///
/// When `outcome` is `Retry`, the agent executes `corrective_actions` before
/// re-evaluating. The `reasoning` field is surfaced in verbose logs.
#[derive(Debug, Deserialize)]
pub struct ReflectionResult {
    pub outcome: ReflectionOutcome,
    pub reasoning: String,
    pub corrective_actions: Vec<ActionRequest>,
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
