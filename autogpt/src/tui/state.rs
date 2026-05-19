// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use {
    crate::cli::session::{SessionStats, TaskStatus},
    crate::cli::settings::GlobalSettings,
    crate::prompts::generic::{
        FOLLOWUP_SYNTHESIS_PROMPT, GENERIC_SYSTEM_PROMPT, IMPLEMENTATION_PLAN_PROMPT,
        INTENT_DETECTION_PROMPT, LESSON_EXTRACTION_PROMPT, REASONING_PROMPT, REFLECTION_PROMPT,
        STATE_SUMMARIZATION_PROMPT, TASK_EXECUTION_PROMPT, TASK_SYNTHESIS_PROMPT,
        WALKTHROUGH_PROMPT,
    },
    crate::tui::theme::{ThemePalette, ThemeVariant},
    serde::{Deserialize, Serialize},
    std::collections::VecDeque,
    std::{
        fs,
        path::PathBuf,
        time::{Duration, Instant},
    },
    tokio::sync::mpsc::UnboundedReceiver,
};

#[cfg(feature = "cli")]
use dirs::home_dir;

/// Events emitted by the agent thread to the TUI.
#[cfg(feature = "cli")]
pub enum TuiEvent {
    /// Append a log message.
    Log(String),
    /// Append text to the last log line.
    LogAppend(String),
    /// Clear the activity log.
    ClearLog,
    /// Update the floating thinking message.
    Thinking(String),
    /// Set a final agent response.
    Response(String),
    /// Update task progress in the right panel.
    TaskUpdate {
        index: usize,
        total: usize,
        description: String,
        status: TaskStatus,
    },
    /// Begin a new agent session, clearing all previous tasks from the panel.
    NewSession,
    /// Update session performance metrics.
    StatsUpdate(SessionStats),
    /// Increment request count.
    IncRequest,
    /// Increment response count.
    IncResponse,
    /// Increment token usage.
    IncTokens { sent: u64, recv: u64 },
    /// Set the agent's current activity label.
    AgentMode(String),
    /// Show interactive session picker with (id, title, status_str) entries.
    SessionsPick(Vec<(String, String, String)>),
    /// Terminate the TUI.
    Quit,
}

/// The five top-level tabs in the TUI.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, strum_macros::EnumIter)]
pub enum AppTab {
    /// Main interaction tab: activity log, task list, and command bar.
    #[default]
    Main,
    /// Token usage charts and session performance metrics.
    Analytics,
    /// Interactive color theme selector.
    Theme,
    /// Runtime configuration settings.
    Settings,
    /// Agent prompt editor with override support.
    Prompts,
}

#[cfg(feature = "cli")]
impl AppTab {
    /// Display label for the tab bar.
    pub fn title(self) -> &'static str {
        match self {
            Self::Main => "  Main  ",
            Self::Analytics => "  Analytics  ",
            Self::Theme => "  Theme  ",
            Self::Settings => "  Settings  ",
            Self::Prompts => "  Prompts  ",
        }
    }

    /// All tabs in display order.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Main,
            Self::Analytics,
            Self::Theme,
            Self::Settings,
            Self::Prompts,
        ]
    }

    /// Advance to the next tab, wrapping around.
    pub fn next(self) -> Self {
        match self {
            Self::Main => Self::Analytics,
            Self::Analytics => Self::Theme,
            Self::Theme => Self::Settings,
            Self::Settings => Self::Prompts,
            Self::Prompts => Self::Main,
        }
    }

    /// Move to the previous tab, wrapping around.
    pub fn prev(self) -> Self {
        match self {
            Self::Main => Self::Prompts,
            Self::Analytics => Self::Main,
            Self::Theme => Self::Analytics,
            Self::Settings => Self::Theme,
            Self::Prompts => Self::Settings,
        }
    }
}

/// A single task row displayed in the main tab task pane.
#[cfg(feature = "cli")]
#[derive(Debug, Clone)]
pub struct TaskRow {
    /// Human-readable task description.
    pub description: String,
    /// Current execution status.
    pub status: TaskStatus,
    /// Zero-based position within the plan.
    pub index: usize,
}

/// Accumulated chart data points for the analytics tab.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, Default)]
pub struct ChartData {
    /// Rolling window of (elapsed_seconds, tokens_sent) samples.
    pub tokens_sent_series: Vec<(f64, f64)>,
    /// Rolling window of (elapsed_seconds, tokens_recv) samples.
    pub tokens_recv_series: Vec<(f64, f64)>,
    /// Number of LLM requests per minute bucket (last N minutes).
    pub requests_per_minute: VecDeque<u64>,
    /// Number of LLM responses per minute bucket (last N minutes).
    pub responses_per_minute: VecDeque<u64>,
    /// Token rate sparkline samples (tokens/30s).
    pub token_rate_sparkline: VecDeque<u64>,
    /// Instant when the session started (used to compute elapsed axis).
    pub session_start: Option<Instant>,
    /// Previous stats snapshot for computing per-tick deltas.
    pub last_stats: SessionStats,
}

impl ChartData {
    /// Record a new stats snapshot and update all chart series.
    pub fn record_stats(&mut self, stats: &SessionStats) {
        if self.session_start.is_none() {
            self.session_start = Some(Instant::now());
        }
        let elapsed = self.session_start.unwrap().elapsed().as_secs_f64();
        self.tokens_sent_series
            .push((elapsed, stats.tokens_sent as f64));
        self.tokens_recv_series
            .push((elapsed, stats.tokens_received as f64));

        let delta_req = stats.requests.saturating_sub(self.last_stats.requests) as u64;
        let delta_res = stats.responses.saturating_sub(self.last_stats.responses) as u64;
        let delta_tok = (stats.tokens_sent + stats.tokens_received)
            .saturating_sub(self.last_stats.tokens_sent + self.last_stats.tokens_received);

        if self.requests_per_minute.len() >= 30 {
            self.requests_per_minute.pop_front();
        }
        if self.responses_per_minute.len() >= 30 {
            self.responses_per_minute.pop_front();
        }
        if self.token_rate_sparkline.len() >= 60 {
            self.token_rate_sparkline.pop_front();
        }

        self.requests_per_minute.push_back(delta_req);
        self.responses_per_minute.push_back(delta_res);
        self.token_rate_sparkline.push_back(delta_tok);
        self.last_stats = stats.clone();
    }
}

/// User-editable overrides for every agent system prompt.
///
/// Serialized to `~/.autogpt/prompts.json`. A `None` value means the
/// built-in default prompt is used.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptOverrides {
    /// Override for the global system instructions.
    pub system_prompt: Option<String>,
    /// Override for the task decomposition prompt.
    pub task_synthesis: Option<String>,
    /// Override for follow-up task synthesis.
    pub followup_synthesis: Option<String>,
    /// Override for the implementation planning prompt.
    pub implementation_plan: Option<String>,
    /// Override for the step-by-step reasoning prompt.
    pub reasoning: Option<String>,
    /// Override for the tool-call execution prompt.
    pub task_execution: Option<String>,
    /// Override for the self-reflection prompt.
    pub reflection: Option<String>,
    /// Override for the lesson-extraction prompt.
    pub lesson_extraction: Option<String>,
    /// Override for the session walkthrough prompt.
    pub walkthrough: Option<String>,
    /// Override for the state summarization prompt.
    pub state_summarization: Option<String>,
    /// Override for intent detection.
    pub intent_detection: Option<String>,
}

impl PromptOverrides {
    /// Load overrides from `~/.autogpt/prompts.json`. Returns defaults on error.
    pub fn load() -> Self {
        let path = home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".autogpt")
            .join("prompts.json");
        if let Ok(raw) = fs::read_to_string(&path) {
            serde_json::from_str(&raw).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Persist overrides to `~/.autogpt/prompts.json`.
    pub fn save(&self) {
        let path = home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".autogpt")
            .join("prompts.json");
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, json);
        }
    }
}

/// Global theme configuration persisted to `~/.autogpt/theme.json`.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// The active color variant.
    pub variant: ThemeVariant,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            variant: ThemeVariant::Dark,
        }
    }
}

impl ThemeConfig {
    /// Load from `~/.autogpt/theme.json`. Falls back to dark theme on error.
    pub fn load() -> Self {
        let path = home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".autogpt")
            .join("theme.json");
        if let Ok(raw) = fs::read_to_string(&path) {
            serde_json::from_str(&raw).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Persist the current theme variant to disk.
    pub fn save(&self) {
        let path = home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".autogpt")
            .join("theme.json");
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, json);
        }
    }

    /// Compute the resolved color palette for the current variant.
    pub fn palette(&self) -> ThemePalette {
        ThemePalette::from_variant(self.variant)
    }
}

/// All mutable state owned by the TUI across frames.
#[cfg(feature = "cli")]
pub struct TuiState {
    /// Currently active tab.
    pub active_tab: AppTab,
    /// Activity log lines (Activity tab).
    pub log_lines: VecDeque<String>,
    /// Streaming thinking lines (Thinking tab).
    pub thinking_lines: VecDeque<String>,
    /// Task list for the right panel.
    pub tasks: Vec<TaskRow>,
    /// Total number of tasks in the plan.
    pub total_tasks: usize,
    /// Session performance metrics.
    pub stats: SessionStats,
    /// Token usage chart data.
    pub chart_data: ChartData,
    /// Input buffer for the command bar.
    pub input_buffer: tui_input::Input,
    /// Mode label (Idle, Thinking, etc.).
    pub agent_mode_label: String,
    /// Theme configuration and colors.
    pub theme_config: ThemeConfig,
    /// Current theme index.
    pub selected_theme_idx: usize,
    /// YOLO setting state.
    pub settings_yolo: bool,
    /// Internet setting state.
    pub settings_internet: bool,
    /// Max retries setting.
    pub settings_max_retries: u8,
    /// Default workspace setting.
    pub settings_workspace: String,
    /// Default provider setting.
    pub settings_provider: String,
    /// Default model setting.
    pub settings_model: String,
    /// Auto-browse setting.
    pub settings_auto_browse: bool,
    /// Verbose logs setting.
    pub settings_verbose: bool,
    /// Metacognition engine toggle.
    pub settings_metacognition: bool,
    /// Input for provider setting.
    pub settings_provider_input: tui_input::Input,
    /// Input for model setting.
    pub settings_model_input: tui_input::Input,
    /// Input for retries setting.
    pub settings_retries_input: tui_input::Input,
    /// Input for workspace setting.
    pub settings_workspace_input: tui_input::Input,
    /// Current focused setting field.
    pub settings_focus_idx: usize,
    /// Global prompt overrides.
    pub prompt_overrides: PromptOverrides,
    /// Selected prompt in Editor.
    pub selected_prompt_idx: usize,
    /// prompt editor input buffer.
    pub prompt_edit_buffer: tui_input::Input,
    /// Whether editing mode is active.
    pub prompt_editing: bool,
    /// Editor scroll position.
    pub prompt_scroll_offset: u16,
    /// Activity log vertical scroll position.
    pub log_scroll_offset: usize,
    /// Activity log horizontal scroll position (chars to skip from left).
    pub log_h_scroll_offset: usize,
    /// Whether the sessions picker overlay is active.
    pub sessions_picking: bool,
    /// Sessions available for picking: (id, title, status_str).
    pub sessions_list: Vec<(String, String, String)>,
    /// Thinking log scroll position.
    pub thinking_scroll_offset: usize,
    /// Task panel scroll position.
    pub task_scroll_offset: usize,
    /// Last received agent response.
    pub last_response: String,
    /// Transient status messages.
    pub status_message: Option<String>,
    /// Exit flag.
    pub should_quit: bool,
    /// Slash command visibility.
    pub slash_autocomplete_active: bool,
    /// Suggestions for slash commands.
    pub slash_matches: Vec<&'static str>,
    /// Index of selected suggestion.
    pub slash_match_idx: usize,
    /// Animation tick counter.
    pub tick_count: u64,
    /// Last time the analytics were sampled.
    pub last_tick_time: Instant,
    /// Event receiver channel.
    pub receiver: UnboundedReceiver<TuiEvent>,
    /// Optional update info: `(current_version, latest_version)`
    pub update_available: Option<(String, String)>,
    /// Whether the user is running AutoGPT in their home directory.
    pub home_dir_warning: bool,
}

#[cfg(feature = "cli")]
impl TuiState {
    /// Construct state from a receiver channel and the current app settings.
    pub fn new(
        receiver: UnboundedReceiver<TuiEvent>,
        settings: &GlobalSettings,
        update_available: Option<(String, String)>,
    ) -> Self {
        let theme_config = ThemeConfig::load();
        let selected_theme_idx = crate::tui::theme::ALL_THEME_VARIANTS
            .iter()
            .position(|v| *v == theme_config.variant)
            .unwrap_or(0);

        let home_dir_warning = matches!((std::env::current_dir(), dirs::home_dir()), (Ok(cwd), Some(home)) if cwd == home);

        Self {
            active_tab: AppTab::Main,
            log_lines: VecDeque::with_capacity(500),
            thinking_lines: VecDeque::with_capacity(200),
            tasks: Vec::new(),
            total_tasks: 0,
            stats: SessionStats::default(),
            chart_data: ChartData::default(),
            input_buffer: tui_input::Input::default(),
            agent_mode_label: "Idle".to_string(),
            theme_config,
            selected_theme_idx,
            settings_yolo: settings.yolo,
            settings_internet: settings.internet_access,
            settings_max_retries: settings.max_retries,
            settings_workspace: settings.workspace.clone(),
            settings_provider: settings.provider.clone(),
            settings_model: settings.model.clone().unwrap_or_default(),
            settings_auto_browse: settings.auto_browse,
            settings_verbose: settings.verbose,
            settings_metacognition: settings.metacognition,
            settings_provider_input: tui_input::Input::default()
                .with_value(settings.provider.clone()),
            settings_model_input: tui_input::Input::default()
                .with_value(settings.model.clone().unwrap_or_default()),
            settings_retries_input: tui_input::Input::default()
                .with_value(settings.max_retries.to_string()),
            settings_workspace_input: tui_input::Input::default()
                .with_value(settings.workspace.clone()),
            settings_focus_idx: 0,
            prompt_overrides: PromptOverrides::load(),
            selected_prompt_idx: 0,
            prompt_edit_buffer: tui_input::Input::default(),
            prompt_editing: false,
            prompt_scroll_offset: 0,
            log_scroll_offset: 0,
            log_h_scroll_offset: 0,
            sessions_picking: false,
            sessions_list: Vec::new(),
            thinking_scroll_offset: 0,
            task_scroll_offset: 0,
            last_response: String::new(),
            status_message: None,
            should_quit: false,
            slash_autocomplete_active: false,
            slash_matches: Vec::new(),
            slash_match_idx: 0,
            tick_count: 0,
            last_tick_time: Instant::now(),
            receiver,
            update_available,
            home_dir_warning,
        }
    }

    /// Strip ANSI escape sequences from a string, returning plain text.
    pub fn strip_ansi(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                if chars.peek() == Some(&'[') {
                    chars.next();
                    for ch in chars.by_ref() {
                        if ch.is_alphabetic() {
                            break;
                        }
                    }
                }
            } else {
                out.push(c);
            }
        }
        out
    }

    /// Strips inline bold/italic markers from a single string, preserving the text content.
    fn strip_inline_markers(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();
        let mut i = 0;
        while i < len {
            if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
                i += 2;
                while i < len {
                    if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
                        i += 2;
                        break;
                    }
                    result.push(chars[i]);
                    i += 1;
                }
            } else if chars[i] == '*' && i + 1 < len && chars[i + 1] != ' ' && i > 0 {
                i += 1;
                while i < len {
                    if chars[i] == '*' {
                        i += 1;
                        break;
                    }
                    result.push(chars[i]);
                    i += 1;
                }
            } else if i + 1 < len && chars[i] == '_' && chars[i + 1] == '_' {
                i += 2;
                while i < len {
                    if i + 1 < len && chars[i] == '_' && chars[i + 1] == '_' {
                        i += 2;
                        break;
                    }
                    result.push(chars[i]);
                    i += 1;
                }
            } else if chars[i] == '_' && i + 1 < len && chars[i + 1] != ' ' && i > 0 {
                i += 1;
                while i < len {
                    if chars[i] == '_' {
                        i += 1;
                        break;
                    }
                    result.push(chars[i]);
                    i += 1;
                }
            } else if chars[i] == '`' {
                i += 1;
                while i < len {
                    if chars[i] == '`' {
                        i += 1;
                        break;
                    }
                    result.push(chars[i]);
                    i += 1;
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        result
    }

    /// Converts a raw markdown string into styled log-line tokens for TUI rendering.
    fn md_to_log_lines(raw: &str) -> Vec<String> {
        let clean = Self::strip_ansi(raw);
        let mut in_code_block = false;
        let mut out = Vec::new();

        for raw_line in clean.split('\n') {
            let line = raw_line.trim_end();

            if line.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }

            if in_code_block {
                out.push(format!("  [code]{line}"));
                continue;
            }

            if let Some(content) = line.strip_prefix("### ") {
                out.push(format!("◆ {}", Self::strip_inline_markers(content)));
            } else if let Some(content) = line.strip_prefix("## ") {
                out.push(format!("◆ {}", Self::strip_inline_markers(content)));
            } else if let Some(content) = line.strip_prefix("# ") {
                out.push(format!("◆ {}", Self::strip_inline_markers(content)));
            } else if let Some(content) = line.strip_prefix("> ") {
                out.push(format!("  [quote]{}", Self::strip_inline_markers(content)));
            } else if line == "---"
                || line == "==="
                || (line.len() >= 3 && line.chars().all(|c| c == '-' || c == '='))
            {
                out.push("─────────────────────────────────".to_string());
            } else if line.starts_with('|') {
                let trimmed = line.trim();
                let is_separator = trimmed
                    .chars()
                    .all(|c| c == '|' || c == '-' || c == ':' || c == ' ');
                if is_separator {
                    out.push("─────────────────────────────────".to_string());
                } else {
                    out.push(format!("  [table]{trimmed}"));
                }
            } else {
                let processed = Self::strip_inline_markers(line);
                out.push(processed);
            }
        }

        out
    }

    /// Appends a markdown string to the activity log, converting it to display lines.
    pub fn push_log(&mut self, line: String) {
        for sub in Self::md_to_log_lines(&line) {
            if self.log_lines.len() >= 1000 {
                self.log_lines.pop_front();
            }
            self.log_lines.push_back(sub);
        }
    }

    /// Appends streamed text to the last log line if it starts with `🤖`; otherwise
    /// pushes a new set of converted display lines.
    pub fn append_log(&mut self, text: String) {
        if self.log_lines.is_empty() {
            self.push_log(text);
            return;
        }

        let lines = Self::md_to_log_lines(&text);
        if lines.is_empty() {
            return;
        }

        if let Some(last) = self.log_lines.back_mut() {
            if last.starts_with("🤖") {
                last.push_str(&lines[0]);
            } else {
                self.log_lines.push_back(lines[0].clone());
            }
        }

        for line in lines.into_iter().skip(1) {
            if self.log_lines.len() >= 1000 {
                self.log_lines.pop_front();
            }
            self.log_lines.push_back(line);
        }
    }

    /// Recompute the slash-command autocomplete list from the current input.
    pub fn update_slash_autocomplete(&mut self) {
        let text = self.input_buffer.value().to_string();
        if text.starts_with('/') && !text.contains(' ') {
            const COMMANDS: &[&str] = &[
                "/help",
                "/clear",
                "/status",
                "/sessions",
                "/workspace",
                "/provider",
                "/models",
                "/mcp",
            ];
            self.slash_matches = COMMANDS
                .iter()
                .copied()
                .filter(|c| c.starts_with(text.as_str()))
                .collect();
            self.slash_autocomplete_active = !self.slash_matches.is_empty() && text.len() > 1;
            if self.slash_match_idx >= self.slash_matches.len() {
                self.slash_match_idx = 0;
            }
        } else {
            self.slash_autocomplete_active = false;
            self.slash_matches.clear();
            self.slash_match_idx = 0;
        }
    }

    /// Append a line to the thinking buffer, capped at 200 entries.
    pub fn push_thinking(&mut self, line: String) {
        if self.thinking_lines.len() >= 200 {
            self.thinking_lines.pop_front();
        }
        self.thinking_lines.push_back(line);
    }

    /// Drain all pending TUI events from the agent channel and apply them.
    pub fn process_events(&mut self) {
        while let Ok(event) = self.receiver.try_recv() {
            match event {
                TuiEvent::Log(s) => self.push_log(s),
                TuiEvent::LogAppend(s) => self.append_log(s),
                TuiEvent::ClearLog => {
                    self.log_lines.clear();
                    self.log_scroll_offset = 0;
                }
                TuiEvent::Thinking(s) => self.push_thinking(s),
                TuiEvent::Response(s) => {
                    self.last_response = s.clone();
                    self.push_log(s);
                    self.agent_mode_label = "Idle".to_string();
                }
                TuiEvent::NewSession => {
                    self.tasks.clear();
                    self.total_tasks = 0;
                    self.task_scroll_offset = 0;
                }
                TuiEvent::TaskUpdate {
                    index,
                    total,
                    description,
                    status,
                } => {
                    self.total_tasks = total.max(self.total_tasks);
                    if let Some(row) = self.tasks.iter_mut().find(|r| r.index == index) {
                        row.status = status;
                        row.description = description;
                    } else {
                        self.tasks.push(TaskRow {
                            description,
                            status,
                            index,
                        });
                    }
                }
                TuiEvent::StatsUpdate(stats) => {
                    self.chart_data.record_stats(&stats);
                    self.stats = stats;
                }
                TuiEvent::IncRequest => {
                    self.stats.requests += 1;
                }
                TuiEvent::IncResponse => {
                    self.stats.responses += 1;
                }
                TuiEvent::IncTokens { sent, recv } => {
                    self.stats.tokens_sent += sent;
                    self.stats.tokens_received += recv;
                }
                TuiEvent::AgentMode(mode) => {
                    self.agent_mode_label = mode;
                }
                TuiEvent::SessionsPick(sessions) => {
                    self.sessions_list = sessions;
                    self.sessions_picking = true;
                    self.push_log(
                        "Type a session number to resume it, or press Esc to cancel:".to_string(),
                    );
                }
                TuiEvent::Quit => {
                    self.should_quit = true;
                }
            }
        }
    }

    /// Apply a theme variant and persist it to disk.
    pub fn apply_theme_variant(&mut self, variant: ThemeVariant) {
        self.theme_config.variant = variant;
        self.theme_config.save();
    }

    /// Ordered display names for all editable prompts.
    pub fn prompt_names() -> &'static [&'static str] {
        &[
            "System Prompt",
            "Task Synthesis",
            "Followup Synthesis",
            "Implementation Plan",
            "Reasoning",
            "Task Execution",
            "Reflection",
            "Lesson Extraction",
            "Walkthrough",
            "State Summarization",
            "Intent Detection",
        ]
    }

    /// Return the effective prompt text for index `idx`, using the override if
    /// present or falling back to the built-in default.
    pub fn get_prompt_text(&self, idx: usize) -> String {
        match idx {
            0 => self
                .prompt_overrides
                .system_prompt
                .clone()
                .unwrap_or_else(|| GENERIC_SYSTEM_PROMPT.to_string()),
            1 => self
                .prompt_overrides
                .task_synthesis
                .clone()
                .unwrap_or_else(|| TASK_SYNTHESIS_PROMPT.to_string()),
            2 => self
                .prompt_overrides
                .followup_synthesis
                .clone()
                .unwrap_or_else(|| FOLLOWUP_SYNTHESIS_PROMPT.to_string()),
            3 => self
                .prompt_overrides
                .implementation_plan
                .clone()
                .unwrap_or_else(|| IMPLEMENTATION_PLAN_PROMPT.to_string()),
            4 => self
                .prompt_overrides
                .reasoning
                .clone()
                .unwrap_or_else(|| REASONING_PROMPT.to_string()),
            5 => self
                .prompt_overrides
                .task_execution
                .clone()
                .unwrap_or_else(|| TASK_EXECUTION_PROMPT.to_string()),
            6 => self
                .prompt_overrides
                .reflection
                .clone()
                .unwrap_or_else(|| REFLECTION_PROMPT.to_string()),
            7 => self
                .prompt_overrides
                .lesson_extraction
                .clone()
                .unwrap_or_else(|| LESSON_EXTRACTION_PROMPT.to_string()),
            8 => self
                .prompt_overrides
                .walkthrough
                .clone()
                .unwrap_or_else(|| WALKTHROUGH_PROMPT.to_string()),
            9 => self
                .prompt_overrides
                .state_summarization
                .clone()
                .unwrap_or_else(|| STATE_SUMMARIZATION_PROMPT.to_string()),
            10 => self
                .prompt_overrides
                .intent_detection
                .clone()
                .unwrap_or_else(|| INTENT_DETECTION_PROMPT.to_string()),
            _ => String::new(),
        }
    }

    /// Commit the prompt editor buffer to the override store and persist it.
    pub fn save_prompt_edit(&mut self) {
        let text = self.prompt_edit_buffer.value().to_string();
        match self.selected_prompt_idx {
            0 => self.prompt_overrides.system_prompt = Some(text),
            1 => self.prompt_overrides.task_synthesis = Some(text),
            2 => self.prompt_overrides.followup_synthesis = Some(text),
            3 => self.prompt_overrides.implementation_plan = Some(text),
            4 => self.prompt_overrides.reasoning = Some(text),
            5 => self.prompt_overrides.task_execution = Some(text),
            6 => self.prompt_overrides.reflection = Some(text),
            7 => self.prompt_overrides.lesson_extraction = Some(text),
            8 => self.prompt_overrides.walkthrough = Some(text),
            9 => self.prompt_overrides.state_summarization = Some(text),
            10 => self.prompt_overrides.intent_detection = Some(text),
            _ => {}
        }
        self.prompt_overrides.save();
        self.prompt_editing = false;
    }

    /// Clear the override for the currently selected prompt, restoring defaults.
    pub fn reset_prompt(&mut self) {
        match self.selected_prompt_idx {
            0 => self.prompt_overrides.system_prompt = None,
            1 => self.prompt_overrides.task_synthesis = None,
            2 => self.prompt_overrides.followup_synthesis = None,
            3 => self.prompt_overrides.implementation_plan = None,
            4 => self.prompt_overrides.reasoning = None,
            5 => self.prompt_overrides.task_execution = None,
            6 => self.prompt_overrides.reflection = None,
            7 => self.prompt_overrides.lesson_extraction = None,
            8 => self.prompt_overrides.walkthrough = None,
            9 => self.prompt_overrides.state_summarization = None,
            10 => self.prompt_overrides.intent_detection = None,
            _ => {}
        }
        self.prompt_overrides.save();
    }

    /// Periodic tick for sampling stats and updating temporal state.
    pub fn tick(&mut self) {
        if self.last_tick_time.elapsed() >= Duration::from_secs(1) {
            self.chart_data.record_stats(&self.stats);
            self.last_tick_time = Instant::now();
        }
    }
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
