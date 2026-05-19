// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Global settings module.
//!
//! Manages the `~/.autogpt/settings.json` file which persists global CLI preferences
//! and all registered MCP server configurations.
//!
//! When the file does not exist it is created with sensible defaults on first access.

use {
    anyhow::{Context, Result},
    dirs::home_dir,
    serde::{Deserialize, Serialize},
    serde_json::{from_str, to_string_pretty},
    std::fs,
    std::path::PathBuf,
};

#[cfg(feature = "mcp")]
use std::collections::HashMap;

// Re-export MCP config types so that CLI command handlers can use a single import path.
#[cfg(all(feature = "cli", feature = "mcp"))]
pub use crate::mcp::settings::{McpOAuthConfig, McpServerConfig, McpTransport};

/// Top-level persistent settings stored at `~/.autogpt/settings.json`.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalSettings {
    /// Mirror of the `--yolo` flag: persists auto-approve preference across sessions.
    #[serde(default)]
    pub yolo: bool,

    /// Last-used session ID. The `--session` flag can override this.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,

    /// Enable Mixture-of-Providers mode by default.
    #[serde(default)]
    pub mixture: bool,

    /// Default AI provider name (e.g. `"gemini"`, `"openai"`).
    #[serde(default = "default_provider")]
    pub provider: String,

    /// Default model name for the active provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Default workspace directory for generated files.
    #[serde(default = "default_workspace")]
    pub workspace: String,

    /// Registered MCP servers keyed by their unique name.
    #[cfg(feature = "mcp")]
    #[serde(default)]
    pub mcp: HashMap<String, McpServerConfig>,

    /// Global MCP policy: names of servers that are allowed (empty = all allowed).
    #[cfg(feature = "mcp")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_allowed: Vec<String>,

    /// Global MCP policy: names of servers that are always excluded.
    #[cfg(feature = "mcp")]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_excluded: Vec<String>,

    /// Enable verbose logging to console.
    #[serde(default)]
    pub verbose: bool,

    /// Maximum number of retry attempts per sub-task before skipping.
    #[serde(default = "default_max_retries")]
    pub max_retries: u8,

    /// Whether to auto-open the browser when the agent produces a runnable app.
    #[serde(default = "default_true")]
    pub auto_browse: bool,

    /// Whether DuckDuckGo web search is enabled during task execution.
    ///
    /// Set to `false` (via `--no-internet`) to disable all outbound search requests.
    /// Persisted so the preference survives restarts without re-passing the flag.
    #[serde(default = "default_true")]
    pub internet_access: bool,

    /// Whether the metacognition engine is enabled during task execution.
    ///
    /// When `true`, the agent performs a periodic self-assessment LLM call after
    /// task groups to refine its strategy. Requires the `mta` feature at compile time
    /// to have any effect.
    #[serde(default = "default_true")]
    pub metacognition: bool,
}

#[cfg(feature = "cli")]
fn default_provider() -> String {
    "gemini".to_string()
}

#[cfg(feature = "cli")]
fn default_workspace() -> String {
    home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".autogpt")
        .join("workspace")
        .to_string_lossy()
        .to_string()
}

#[cfg(feature = "cli")]
fn default_max_retries() -> u8 {
    3
}

#[cfg(feature = "cli")]
fn default_true() -> bool {
    true
}

#[cfg(feature = "cli")]
impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            yolo: false,
            session: None,
            mixture: false,
            provider: default_provider(),
            model: None,
            workspace: default_workspace(),
            #[cfg(feature = "mcp")]
            mcp: HashMap::new(),
            #[cfg(feature = "mcp")]
            mcp_allowed: Vec::new(),
            #[cfg(feature = "mcp")]
            mcp_excluded: Vec::new(),
            verbose: false,
            max_retries: default_max_retries(),
            auto_browse: true,
            internet_access: true,
            metacognition: true,
        }
    }
}

/// Manages loading and persisting `GlobalSettings` from `~/.autogpt/settings.json`.
#[cfg(feature = "cli")]
pub struct SettingsManager {
    path: PathBuf,
}

#[cfg(feature = "cli")]
impl SettingsManager {
    /// Creates a `SettingsManager` targeting the default `~/.autogpt/settings.json` path.
    pub fn new() -> Self {
        let path = home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".autogpt")
            .join("settings.json");
        Self { path }
    }

    /// Creates a `SettingsManager` targeting an explicit file path.
    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    /// Loads settings from disk, creating the file with defaults when absent.
    pub fn load(&self) -> Result<GlobalSettings> {
        if !self.path.exists() {
            let defaults = GlobalSettings::default();
            self.save(&defaults)?;
            return Ok(defaults);
        }
        let raw = fs::read_to_string(&self.path)
            .with_context(|| format!("Reading settings from {}", self.path.display()))?;
        from_str(&raw).with_context(|| format!("Parsing settings from {}", self.path.display()))
    }

    /// Writes `settings` to disk as pretty-printed JSON.
    pub fn save(&self, settings: &GlobalSettings) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Creating dir {}", parent.display()))?;
        }
        let json = to_string_pretty(settings).context("Serializing settings to JSON")?;
        fs::write(&self.path, json)
            .with_context(|| format!("Writing settings to {}", self.path.display()))
    }

    /// Returns the filesystem path of the settings file.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Adds or replaces an MCP server in settings and persists the result.
    #[cfg(feature = "mcp")]
    pub fn add_mcp_server(&self, config: McpServerConfig) -> Result<GlobalSettings> {
        let mut settings = self.load()?;
        settings.mcp.insert(config.name.clone(), config);
        self.save(&settings)?;
        Ok(settings)
    }

    /// Removes an MCP server by name and persists. Returns `true` when the server existed.
    #[cfg(feature = "mcp")]
    pub fn remove_mcp_server(&self, name: &str) -> Result<(GlobalSettings, bool)> {
        let mut settings = self.load()?;
        let existed = settings.mcp.remove(name).is_some();
        self.save(&settings)?;
        Ok((settings, existed))
    }
}

#[cfg(feature = "cli")]
impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
