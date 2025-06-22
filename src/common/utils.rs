//! # Utils module.
//!
//! This module provides various utility functions and common structures that can be used across different parts of the project.
//!
//! ## Structures
//!
//! - `Communication`: Represents a communication messages between agents.
//! - `Status`: Represents the status of an agent.
//! - `Route`: Represents a route object.
//! - `Scope`: Represents the scope of a project.
//! - `Tasks`: Represents a fact tasks.
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
//! use autogpt::common::utils::{Communication, Status, Route, Scope, Tasks, extract_json_string, extract_array, similarity, strip_code_blocks};
//!
//! let communication = Communication {
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
//! let tasks = Tasks {
//!     description: "This is a task description.".into(),
//!     scope: Some(scope),
//!     urls: Some(vec!["https://kevin-rs.dev".into()]),
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
use crate::agents::agent::AgentGPT;
#[cfg(feature = "cli")]
use crate::traits::agent::Agent;
#[cfg(feature = "cli")]
use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::env::var;
#[cfg(feature = "cli")]
use std::{io, io::Read, process::Command, process::Stdio};
#[cfg(feature = "cli")]
use webbrowser::{Browser, BrowserOptions, open_browser_with_options};
#[cfg(feature = "cli")]
use {
    tracing::{error, info, warn},
    tracing_appender::rolling,
    tracing_subscriber::Layer,
    tracing_subscriber::Registry,
    tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt,
    tracing_subscriber::{filter, fmt},
};

#[cfg(feature = "gem")]
use gems::{
    Client as GeminiClient,
    messages::{Content, Message as GeminiMessage},
    models::Model as GeminiModel,
    traits::CTrait,
};
#[cfg(feature = "oai")]
use openai_dive::v1::{
    api::Client as OpenAIClient,
    models::FlagshipModel,
    resources::chat::{ChatMessage, ChatMessageContent},
};

#[cfg(feature = "cld")]
use anthropic_ai_sdk::{
    client::AnthropicClient,
    types::message::{Message as AnthMessage, MessageError},
};

/// Enum representing supported AI clients.
#[derive(Debug, Clone)]
pub enum ClientType {
    /// OpenAI client.
    #[cfg(feature = "oai")]
    OpenAI(OpenAIClient),

    /// Google Gemini client.
    #[cfg(feature = "gem")]
    Gemini(GeminiClient),

    /// Anthropic Gemini client.
    #[cfg(feature = "cld")]
    Anthropic(AnthropicClient),
}

impl Default for ClientType {
    fn default() -> Self {
        ClientType::from_env()
    }
}

impl ClientType {
    pub fn from_env() -> Self {
        let provider = var("AI_PROVIDER").unwrap_or_else(|_| "gemini".to_string());

        #[cfg(feature = "oai")]
        if provider == "openai" {
            let openai_client = OpenAIClient::new_from_env();
            return ClientType::OpenAI(openai_client);
        }

        #[cfg(feature = "gem")]
        if provider == "gemini" || cfg!(not(feature = "oai")) {
            let model = var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.0-flash".to_string());
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

        #[allow(unreachable_code)]
        {
            panic!(
                "Invalid AI_PROVIDER `{}` or missing required feature flags. \
                Make sure to enable at least one of: `oai`, `gem`, `cld`.",
                provider
            );
        }
    }
}

/// Represents a communication between agents.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Communication {
    /// The role of the communication.
    pub role: Cow<'static, str>,
    /// The content of the communication.
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
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
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
pub struct Scope {
    /// Indicates if CRUD operations are required.
    pub crud: bool,
    /// Indicates if user login and logout are required.
    pub auth: bool,
    /// Indicates if external URLs are required.
    pub external: bool,
}

/// Represents a fact tasks.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct Tasks {
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

impl Tasks {
    pub fn from_payload(payload: &str) -> Self {
        Tasks {
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
    // Check if the text starts with '[' and ends with ']'
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

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for (i, item) in matrix.iter_mut().enumerate().take(len1 + 1) {
        item[0] = i;
    }

    for j in 0..=len2 {
        matrix[0][j] = j;
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
) -> Result<Option<std::process::Child>, Box<dyn std::error::Error + Send + Sync>> {
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

        _ => Err(format!("Unsupported language: {}", language).into()),
    }
}

#[cfg(feature = "cli")]
pub fn setup_logging() -> anyhow::Result<()> {
    let file_appender = rolling::daily("logs", "autogpt_log");

    let console_layer = fmt::Layer::new()
        .compact()
        .without_time()
        .with_file(false)
        .with_line_number(false)
        .with_thread_ids(false)
        .with_target(false)
        .with_writer(std::io::stdout)
        .with_filter(filter::LevelFilter::INFO);

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
            "[*] \"AGI\": 🤔 Thinking... Maybe it's time to run the application? (yes/no)"
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
                    error!(
                        "{}",
                        format!("[*] \"AGI\": Error: {}", e).bright_red().bold()
                    );
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
    /// OpenAI model.
    #[cfg(feature = "oai")]
    OpenAI(FlagshipModel),

    /// Google Gemini model.
    #[cfg(feature = "gem")]
    Gemini(GeminiModel),

    /// Anthropic claude model.
    #[cfg(feature = "cld")]
    Claude(String), // Example: "claude-3-7-sonnet-latest"
}

impl Default for Model {
    fn default() -> Self {
        #[cfg(feature = "oai")]
        {
            Model::OpenAI(FlagshipModel::Gpt4O)
        }

        #[cfg(all(not(feature = "oai"), feature = "cld"))]
        {
            return Model::Claude("claude-3-7-sonnet-latest".to_string());
        }

        #[cfg(all(not(any(feature = "oai", feature = "cld")), feature = "gem"))]
        {
            return Model::Gemini(GeminiModel::Flash20);
        }

        #[cfg(not(any(feature = "oai", feature = "gem", feature = "cld")))]
        {
            panic!(
                "At least one of the features `oai`, `gem`, or `cld` must be enabled for Model::default()"
            );
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    /// OpenAI message type.
    #[cfg(feature = "oai")]
    OpenAI(ChatMessage),

    /// Google message type.
    #[cfg(feature = "gem")]
    Gemini(GeminiMessage),

    /// Anthropic claude type.
    #[cfg(feature = "cld")]
    Claude(AnthMessage),
}

impl Default for Message {
    fn default() -> Self {
        #[cfg(feature = "oai")]
        {
            Message::OpenAI(ChatMessage::User {
                content: ChatMessageContent::Text("Hello".into()),
                name: None,
            })
        }

        #[cfg(all(not(feature = "oai"), feature = "cld"))]
        {
            return Message::Claude(AnthMessage::new_text(Role::User, "Hello"));
        }

        #[cfg(all(not(any(feature = "oai", feature = "cld")), feature = "gem"))]
        {
            return Message::Gemini(GeminiMessage::User {
                content: Content::Text("Hello".into()),
                name: None,
            });
        }

        #[cfg(not(any(feature = "oai", feature = "gem", feature = "cld")))]
        {
            panic!(
                "At least one of the features `oai`, `gem`, or `cld` must be enabled for Message::default()"
            );
        }
    }
}

impl Message {
    pub fn from_text(_text: impl Into<String>) -> Self {
        #[cfg(feature = "oai")]
        {
            Message::OpenAI(ChatMessage::User {
                content: ChatMessageContent::Text(_text.into()),
                name: None,
            })
        }

        #[cfg(all(not(feature = "oai"), feature = "cld"))]
        {
            return Message::Claude(AnthMessage::new_text(Role::User, _text.into()));
        }

        #[cfg(all(not(any(feature = "oai", feature = "cld")), feature = "gem"))]
        {
            return Message::Gemini(GeminiMessage::User {
                content: Content::Text(_text.into()),
                name: None,
            });
        }

        #[cfg(not(any(feature = "oai", feature = "gem", feature = "cld")))]
        {
            panic!(
                "At least one of the features `oai`, `gem`, or `cld` must be enabled for Message::from_text()"
            );
        }
    }
}

#[derive(Debug, PartialEq, Default, Clone)]
pub enum Tool {
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

    /// Communication & Scheduling
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

    Optimize,
    Frontend,
    Backend,

    /// Custom / Plugin Tool
    Plugin(String),
}
