// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "cli")]
use crate::cli::session::SessionStats;
#[cfg(all(feature = "cli", feature = "mcp"))]
use crate::mcp::settings::McpServerConfig;
#[cfg(all(feature = "cli", feature = "mcp"))]
use crate::mcp::types::{McpServerInfo, McpServerStatus};
#[cfg(feature = "cli")]
use crate::tui::state::TuiEvent;
#[cfg(feature = "cli")]
use colored::Colorize;
#[cfg(feature = "cli")]
use dialoguer::{Select, theme::ColorfulTheme};
#[cfg(feature = "cli")]
use indicatif::{ProgressBar, ProgressStyle};
#[cfg(feature = "cli")]
use std::time::Duration;
#[cfg(feature = "cli")]
use termimad::{MadSkin, crossterm::style::Color};
#[cfg(feature = "cli")]
use tokio::sync::mpsc::UnboundedSender;
#[cfg(feature = "cli")]
use tracing::{error, info, warn};

/// Returns the true display column width of a string in a terminal.
///
/// Uses `unicode-width` to correctly measure wide characters (CJK, emoji)
/// as 2 columns and combining characters as 0 columns, matching the actual
/// terminal cursor advance for cursor positioning.
#[cfg(feature = "cli")]
pub fn display_width(s: &str) -> usize {
    use unicode_width::UnicodeWidthStr;
    UnicodeWidthStr::width(s)
}

/// Returns the emoji string on Unix-like platforms and the ASCII fallback on Windows.
///
/// Windows terminals (Console Host, older ConPTY builds) may display multi-codepoint
/// emoji as empty boxes. This helper selects the safe fallback automatically so UI
/// chrome (status badges, section markers) renders correctly on all platforms.
#[cfg(feature = "cli")]
pub fn emoji_text<'a>(emoji: &'a str, fallback: &'a str) -> &'a str {
    #[cfg(windows)]
    {
        let _ = emoji;
        fallback
    }
    #[cfg(not(windows))]
    {
        let _ = fallback;
        emoji
    }
}

/// Terminal width used for box drawings.
#[cfg(feature = "cli")]
const BOX_WIDTH: usize = 80;

#[cfg(feature = "cli")]
pub use crate::cli::models::ProviderModel;

/// Status variants for task progress indicators.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

/// Prints the AutoGPT gradient pixel-art ASCII banner.
///
/// Each column of the banner is coloured across a hot-pink → lavender → cyan gradient
/// using `colored`'s true-colour support, giving visual depth similar to the Gemini CLI.
#[cfg(feature = "cli")]
pub fn print_banner() {
    let logo_lines = [
        " █████╗ ██╗   ██╗████████╗ ██████╗  ██████╗ ██████╗ ████████╗",
        "██╔══██╗██║   ██║╚══██╔══╝██╔═══██╗██╔════╝ ██╔══██╗╚══██╔══╝",
        "███████║██║   ██║   ██║   ██║   ██║██║  ███╗██████╔╝   ██║   ",
        "██╔══██║██║   ██║   ██║   ██║   ██║██║   ██║██╔═══╝    ██║   ",
        "██║  ██║╚██████╔╝   ██║   ╚██████╔╝╚██████╔╝██║        ██║   ",
        "╚═╝  ╚═╝ ╚═════╝    ╚═╝    ╚═════╝  ╚═════╝ ╚═╝        ╚═╝   ",
    ];

    let gradient_stops: &[(u8, u8, u8)] = &[
        (255, 80, 180),
        (220, 110, 200),
        (180, 140, 230),
        (140, 170, 245),
        (100, 210, 250),
        (60, 230, 240),
    ];

    info!("");
    for (line_idx, line) in logo_lines.iter().enumerate() {
        let (r, g, b) = gradient_stops[line_idx % gradient_stops.len()];
        info!("{}", line.truecolor(r, g, b).bold());
    }
    info!("");
}

/// Prints the startup tips greeting block.
#[cfg(feature = "cli")]
pub fn print_greeting() {
    info!("{}", "Tips for getting started:".bold());
    info!(
        "  {}. Describe your project; AutoGPT will decompose and execute it autonomously.",
        "1".bright_cyan()
    );
    info!(
        "  {}. Review and approve the generated plan before execution begins.",
        "2".bright_cyan()
    );
    info!(
        "  {}. Use {} to list all available commands.",
        "3".bright_cyan(),
        "/help".bright_magenta().bold()
    );
    info!(
        "  {}. Use {} for unattended, continuous execution.",
        "4".bright_cyan(),
        "-y / --yolo".bright_yellow()
    );
    info!(
        "  {}. Press {} during execution to interrupt!",
        "5".bright_cyan(),
        "ESC".yellow().bold()
    );
    info!("");
}

/// Renders the help table for all available slash commands to the TUI log.
#[cfg(feature = "cli")]
pub fn render_help_table_to_log(tx: &UnboundedSender<TuiEvent>) {
    let commands: &[(&str, &str)] = &[
        (
            "/help",
            "Display this help table with all available commands",
        ),
        (
            "/sessions",
            "List and interactively resume a previous session",
        ),
        ("/models", "List available models for the current provider"),
        ("/clear", "Clear the Activity Log"),
        (
            "/status",
            "Show current session info, task progress, and model",
        ),
        ("/workspace", "Display the current workspace directory path"),
        ("/provider", "Show and switch the active LLM provider"),
        ("/mcp", "List, inspect, and manage MCP servers"),
        ("exit / quit", "Save the current session and exit AutoGPT"),
    ];

    let _ = tx.send(TuiEvent::Log("Available Commands:".to_string()));
    for (cmd, desc) in commands {
        let _ = tx.send(TuiEvent::Log(format!("  {:14}  {}", cmd, desc)));
    }

    let _ = tx.send(TuiEvent::Log("".to_string()));
    let _ = tx.send(TuiEvent::Log("TUI Navigation:".to_string()));
    let _ = tx.send(TuiEvent::Log(
        "  Up / Down       Scroll Activity Log".to_string(),
    ));
    let _ = tx.send(TuiEvent::Log(
        "  PgUp / PgDn     Scroll Activity Log (fast)".to_string(),
    ));
    let _ = tx.send(TuiEvent::Log(
        "  Ctrl+Up/Down    Scroll Task List (right panel)".to_string(),
    ));
    let _ = tx.send(TuiEvent::Log("  Tab / S-Tab     Switch Tabs".to_string()));
    let _ = tx.send(TuiEvent::Log(
        "  Esc             Interrupt / Abort".to_string(),
    ));

    render_mcp_help_entries_to_log(tx);
}

/// Renders the help table for all available slash commands.
#[cfg(feature = "cli")]
pub fn render_help_table() {
    info!("");
    info!("{}", "Available Commands".bright_cyan().bold());
    info!("{}", "─".repeat(BOX_WIDTH).bright_black());

    let commands: &[(&str, &str)] = &[
        (
            "/help",
            "Display this help table with all available commands",
        ),
        (
            "/sessions",
            "List and interactively resume a previous session",
        ),
        (
            "/models",
            "List available models for the current provider and switch",
        ),
        ("/clear", "Clear the screen and reprint the AutoGPT banner"),
        (
            "/status",
            "Show current session info, task progress, and model",
        ),
        ("/workspace", "Display the current workspace directory path"),
        ("/provider", "Show and switch the active LLM provider"),
        ("exit / quit", "Save the current session and exit AutoGPT"),
    ];

    for (cmd, desc) in commands {
        info!("  {:<15}  {}", cmd.bright_magenta().bold(), desc.white());
    }

    info!("{}", "─".repeat(BOX_WIDTH).bright_black());
    #[cfg(feature = "cli")]
    render_mcp_help_entries();
    info!("");
}

/// Renders an interactive model selector using `dialoguer`.
#[cfg(feature = "cli")]
pub fn render_model_selector(models: &[ProviderModel], current_idx: usize) -> usize {
    info!("");
    info!("{}", "Select Model".bright_cyan().bold());
    info!("{}", "─".repeat(BOX_WIDTH).bright_black());

    let labels: Vec<String> = models
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let bullet = if i == current_idx { "●" } else { " " };
            if m.description.is_empty() {
                format!("{} {}", bullet, m.display_name)
            } else {
                format!("{} {} - {}", bullet, m.display_name, m.description)
            }
        })
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&labels)
        .default(current_idx)
        .interact()
        .unwrap_or(current_idx);

    info!("{}", "─".repeat(BOX_WIDTH).bright_black());
    selection
}

/// Renders markdown content using `termimad`, applying syntax colour to code blocks.
///
/// `termimad` writes directly to stdout - this is intentional for rich terminal output
/// that cannot be easily proxied through `tracing`.
#[cfg(feature = "cli")]
pub fn render_markdown(content: &str) {
    let mut skin = MadSkin::default();
    skin.code_block.set_fg(Color::Cyan);
    skin.inline_code.set_fg(Color::Cyan);
    skin.bold.set_fg(Color::White);
    skin.print_text(content);
}

/// Creates and starts a themed spinner with autogpt-inspired tick animation.
#[cfg(feature = "cli")]
pub fn create_spinner(message: &str, hidden: bool) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    if hidden {
        spinner.set_draw_target(indicatif::ProgressDrawTarget::hidden());
    }
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.magenta} {msg:.cyan}")
            .unwrap()
            .tick_strings(&[
                "⟳ synthesizing ",
                "⟲ processing  ",
                "↻ computing   ",
                "↺ reasoning   ",
                "⟳ executing   ",
                "⟲ reflecting  ",
                "↻ verifying   ",
                "↺ finalizing  ",
            ]),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner
}

/// Logs a task item with a coloured status icon via `tracing::info!`.
#[cfg(feature = "cli")]
pub fn print_task_item(description: &str, status: TaskStatus) {
    let (icon, coloured) = match status {
        TaskStatus::Pending => ("○", format!("○ {}", description).bright_black().to_string()),
        TaskStatus::InProgress => (
            "●",
            format!("● {}", description)
                .bright_yellow()
                .bold()
                .to_string(),
        ),
        TaskStatus::Completed => (
            "✓",
            format!("✓ {}", description)
                .bright_green()
                .bold()
                .to_string(),
        ),
        TaskStatus::Failed => (
            "✗",
            format!("✗ {}", description).bright_red().bold().to_string(),
        ),
        TaskStatus::Skipped => ("⊘", format!("⊘ {}", description).bright_black().to_string()),
    };
    let _ = icon;
    info!("  {}", coloured);
}

/// Logs a section header with a coloured divider via `tracing::info!`.
#[cfg(feature = "cli")]
pub fn print_section(title: &str) {
    info!("");
    info!("{} {}", "▸".bright_cyan(), title.bold().white());
}

/// Logs an agent-tagged message via `tracing::info!`.
#[cfg(feature = "cli")]
pub fn print_agent_msg(tag: &str, message: &str) {
    info!(
        "{} {}",
        format!("[AutoGPT·{}]", tag).bright_magenta().bold(),
        message.white()
    );
}

/// Logs a warning message via `tracing::warn!` with yellow colouring.
#[cfg(feature = "cli")]
pub fn print_warning(message: &str) {
    warn!("{}  {}", "⚠".bright_yellow(), message.bright_yellow());
}

/// Logs an error message via `tracing::error!` with red colouring.
#[cfg(feature = "cli")]
pub fn print_error(message: &str) {
    error!("{}  {}", "✗".bright_red(), message.bright_red());
}

/// Logs a success message via `tracing::info!` with green colouring.
#[cfg(feature = "cli")]
pub fn print_success(message: &str) {
    info!("{}  {}", "✓".bright_green(), message.bright_green().bold());
}

/// Log a status bar line to the terminal.
#[cfg(feature = "cli")]
pub fn print_status_bar(cwd: &str, model: &str, provider: &str) {
    info!(
        "  {}  {}  {}",
        cwd.bright_blue(),
        model.bright_magenta(),
        provider.bright_black()
    );
}

/// Builds a compact ANSI status segment string from session stats for use in the TUI status line.
///
/// Format: `↑{req}req ↓{resp}res ~{tokens_sent}tk↑ ~{tokens_recv}tk↓  🌐`
/// The web-search badge is only appended when `internet_access` is true.
#[cfg(feature = "cli")]
pub fn build_stats_status_segment(stats: &SessionStats, internet_access: bool) -> String {
    let search_badge = if internet_access {
        "  \x1b[94m🌐 live-search\x1b[0m".to_string()
    } else {
        String::new()
    };
    format!(
        "\x1b[36m↑{}req\x1b[0m \x1b[36m↓{}res\x1b[0m  \x1b[33m~{}tk↑\x1b[0m \x1b[32m~{}tk↓\x1b[0m{}",
        stats.requests, stats.responses, stats.tokens_sent, stats.tokens_received, search_badge
    )
}

/// Renders a summary table of all registered MCP servers.
#[cfg(all(feature = "cli", feature = "mcp"))]
pub fn render_mcp_list(infos: &[McpServerInfo]) {
    print_section("MCP Servers");
    if infos.is_empty() {
        print_warning("No MCP servers configured. Use `autogpt mcp add` or `/mcp add`.");
        return;
    }
    let col_name = 20usize;
    let col_status = 14usize;
    let col_tools = 8usize;
    info!(
        "  {:<col_name$}  {:<col_status$}  {:<col_tools$}  Description",
        "Name".bold(),
        "Status".bold(),
        "Tools".bold(),
    );
    info!("  {}", "─".repeat(BOX_WIDTH - 2));
    for info in infos {
        let status_str = match info.status {
            McpServerStatus::Connected => "● connected".bright_green().bold().to_string(),
            McpServerStatus::Connecting => "⟳ connecting".bright_yellow().to_string(),
            McpServerStatus::Disconnected => "○ offline".bright_red().to_string(),
        };
        let tool_count = info.tools.len().to_string();
        let desc = info.description.chars().take(40).collect::<String>();
        info!(
            "  {:<col_name$}  {:<col_status$}  {:<col_tools$}  {}",
            info.name.bright_magenta().bold(),
            status_str,
            tool_count.bright_cyan(),
            desc.bright_black(),
        );
        if let Some(ref err) = info.error {
            info!(
                "    {}  {}",
                "⚠".bright_yellow(),
                err.to_string().bright_red()
            );
        }
    }
    info!("");
}

/// Renders detailed information about a single MCP server, including its tools.
#[cfg(all(feature = "cli", feature = "mcp"))]
pub fn render_mcp_inspect(info: &McpServerInfo, config: &McpServerConfig) {
    print_section(&format!("MCP Server: {}", info.name));
    info!(
        "  {}  {}",
        "Transport:".bright_black(),
        config.transport.to_string().bright_cyan()
    );
    info!(
        "  {}  {}",
        "Connection:".bright_black(),
        config.connection_display().bright_white()
    );
    info!(
        "  {}  {}",
        "Status:   ".bright_black(),
        info.status.to_string().bright_cyan()
    );
    if let Some(ref desc) = config.description {
        info!(
            "  {}  {}",
            "Description:".bright_black(),
            desc.to_string().bright_white()
        );
    }
    info!(
        "  {}  {}",
        "Trust:    ".bright_black(),
        if config.trust {
            "yes".bright_green()
        } else {
            "no".bright_black()
        }
    );
    if config.timeout_ms != 500_000 {
        info!(
            "  {}  {}ms",
            "Timeout:  ".bright_black(),
            config.timeout_ms.to_string().bright_cyan()
        );
    }
    if let Some(ref err) = info.error {
        print_error(&format!("Last connection error: {err}"));
    }

    if info.tools.is_empty() {
        print_warning("No tools discovered (server may be offline or has no tools).");
    } else {
        print_section(&format!("Available Tools ({}):", info.tools.len()));
        for tool in &info.tools {
            info!(
                "  {}  {}",
                tool.name.bright_magenta().bold(),
                tool.description.bright_black()
            );
            for (param_name, param) in &tool.params {
                let req = if param.required {
                    "*".bright_red().bold().to_string()
                } else {
                    " ".to_string()
                };
                info!(
                    "    {} {}: {} - {}",
                    req,
                    param_name.bright_cyan(),
                    param.param_type.bright_black(),
                    param.description.bright_black(),
                );
            }
        }
    }
    info!("");
}

/// Append MCP server list to the TUI log.
#[cfg(all(feature = "cli", feature = "mcp"))]
pub fn render_mcp_list_to_log(tx: &UnboundedSender<TuiEvent>, infos: &[McpServerInfo]) {
    use crate::mcp::types::McpServerStatus;
    let _ = tx.send(TuiEvent::Log("=== MCP Servers ===".to_string()));
    if infos.is_empty() {
        let _ = tx.send(TuiEvent::Log("No MCP servers configured.".to_string()));
        return;
    }
    for info in infos {
        let status = match info.status {
            McpServerStatus::Connected => "connected",
            McpServerStatus::Connecting => "connecting",
            McpServerStatus::Disconnected => "offline",
        };
        let _ = tx.send(TuiEvent::Log(format!(
            "  {} [{}] - {} tools",
            info.name,
            status,
            info.tools.len()
        )));
    }
}

/// Append MCP server inspect info to the TUI log.
#[cfg(all(feature = "cli", feature = "mcp"))]
pub fn render_mcp_inspect_to_log(
    tx: &UnboundedSender<TuiEvent>,
    info: &McpServerInfo,
    config: &McpServerConfig,
) {
    let _ = tx.send(TuiEvent::Log(format!("=== MCP Inspect: {} ===", info.name)));
    let _ = tx.send(TuiEvent::Log(format!("  Transport:  {}", config.transport)));
    let _ = tx.send(TuiEvent::Log(format!(
        "  Connection: {}",
        config.connection_display()
    )));
    let _ = tx.send(TuiEvent::Log(format!("  Status:     {}", info.status)));
    if let Some(ref desc) = config.description {
        let _ = tx.send(TuiEvent::Log(format!("  Description:{}", desc)));
    }
    let _ = tx.send(TuiEvent::Log(format!("  Tools ({}):", info.tools.len())));
    for tool in &info.tools {
        let _ = tx.send(TuiEvent::Log(format!(
            "    - {}: {}",
            tool.name, tool.description
        )));
    }
}

/// Appends MCP entries to the help table for the TUI log.
#[cfg(feature = "cli")]
pub fn render_mcp_help_entries_to_log(tx: &UnboundedSender<TuiEvent>) {
    let _ = tx.send(TuiEvent::Log("MCP Commands:".to_string()));
    let _ = tx.send(TuiEvent::Log(
        "  /mcp list           List servers".to_string(),
    ));
    let _ = tx.send(TuiEvent::Log(
        "  /mcp inspect <name> Show tools".to_string(),
    ));
    let _ = tx.send(TuiEvent::Log(
        "  /mcp remove <name>  Remove server".to_string(),
    ));
    let _ = tx.send(TuiEvent::Log("  /mcp call <srv> <tool> [args]".to_string()));
}

/// Appends MCP entries to the help table rendered by `/help`.
#[cfg(feature = "cli")]
pub fn render_mcp_help_entries() {
    info!("");
    info!("{}", "MCP Commands".bold().bright_magenta());
    info!(
        "  {}  {}",
        "/mcp list".bright_cyan().bold(),
        "Show all configured MCP servers and their status".bright_black()
    );
    info!(
        "  {}  {}",
        "/mcp inspect <name>".bright_cyan().bold(),
        "Inspect a server and list its tools".bright_black()
    );
    info!(
        "  {}  {}",
        "/mcp remove <name>".bright_cyan().bold(),
        "Remove a server registration".bright_black()
    );
    info!(
        "  {}  {}",
        "/mcp call <srv> <tool> [args]".bright_cyan().bold(),
        "Call an MCP tool with JSON args or key=val pairs".bright_black()
    );
    info!("");
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
