// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "cli")]
pub mod ast;
#[cfg(feature = "cli")]
pub mod commands;
#[cfg(feature = "cli")]
pub mod generator;
#[cfg(feature = "cli")]
pub mod parser;
#[cfg(feature = "cli")]
pub mod utils;

#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};

#[cfg(feature = "cli")]
use clap::Subcommand as ClapSubcommand;
#[cfg(feature = "cli")]
use clap::builder::styling::{AnsiColor, Effects, Styles};

/// Defines custom styles for CLI output.
#[cfg(feature = "cli")]
fn styles() -> Styles {
    // Define styles for different CLI elements
    Styles::styled()
        .header(AnsiColor::Red.on_default() | Effects::BOLD)
        .usage(AnsiColor::Red.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .error(AnsiColor::Red.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Green.on_default())
}

/// Represents the command-line interface (CLI) for autogpt.
#[cfg(feature = "cli")]
#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    name = "autogpt",
    propagate_version = true,
    styles = styles(),
    help_template = r#"{before-help}{name} {version}
{about}
{usage-heading} {usage}
{all-args}{after-help}

AUTHORS:
    {author}
"#,
    about=r#"
 █████  ██    ██ ████████  ██████   ██████  ██████  ████████ 
██   ██ ██    ██    ██    ██    ██ ██       ██   ██    ██    
███████ ██    ██    ██    ██    ██ ██   ███ ██████     ██    
██   ██ ██    ██    ██    ██    ██ ██    ██ ██         ██    
██   ██  ██████     ██     ██████   ██████  ██         ██    

The `autogpt` CLI enables interaction with AI providers through a suite of
built-in, specialized autonomous AI agents designed for various stages of
project development.

Modes of Operation:
-------------------
Autogpt supports 4 modes:

1. Interactive Mode (default):
   Run `autogpt` with no arguments to launch the GenericGPT interactive
   shell with session persistence, model switching, and multi-provider
   support.

2. Direct Prompt Mode:
   Use `autogpt -p "<prompt>"` to send a single prompt directly to the
   active LLM provider and stream the response, no agentic workflow
   required.

3. Agentic Networkless Mode:
   Run `autogpt <subcommand>` (e.g. `back`, `front`, `arch`) to invoke a
   specialized autonomous agent locally without an orchestrator.

4. Agentic Networking Mode:
   Use `autogpt --net` to connect to an `orchgpt` orchestrator over
   TLS-encrypted TCP for distributed multi-agent collaboration.
"#

)]
pub struct Cli {
    #[clap(global = true, short, long)]
    pub verbose: bool,

    /// Directly prompt the LLM without using an agentic workflow.
    ///
    /// Sends a single, raw prompt to the LLM for immediate response, bypassing the
    /// full agentic session lifecycle.
    #[arg(short, long)]
    pub prompt: Option<String>,

    /// Connect to an `orchgpt` orchestrator over TLS (networking mode).
    ///
    /// When set, `autogpt` connects to the configured `ORCHESTRATOR_ADDRESS` instead
    /// of launching the interactive generic agent.
    #[arg(long, default_value_t = false)]
    pub net: bool,

    /// Auto-approve all plans and execute without user confirmation (YOLO mode).
    ///
    /// Suppresses the approval gate between plan generation and task execution.
    /// Ideal for unattended pipelines and CI environments.
    #[arg(short, long, default_value_t = false)]
    pub yolo: bool,

    /// Disable DuckDuckGo web search during task execution.
    ///
    /// By default the agent may perform web searches to gather information
    /// needed to complete tasks. Pass this flag to restrict all network access.
    #[arg(long, default_value_t = false)]
    pub no_internet: bool,

    /// Resume a specific previous session by its session ID.
    ///
    /// The session ID is the UUID shown in `/sessions`. When provided, AutoGPT
    /// loads the session history and resumes from where it left off.
    #[arg(short, long, value_name = "SESSION_ID")]
    pub session: Option<String>,

    /// Enable Mixture of Providers mode.
    ///
    /// Fans each prompt out to all configured AI providers concurrently and
    /// returns the highest-scored response. Only applies in interactive TUI mode.
    /// Requires the `mop` feature flag at compile time.
    #[cfg(feature = "mop")]
    #[arg(short = 'm', long, default_value_t = false)]
    pub mixture: bool,

    /// Enable collaborative multi-provider mode.
    ///
    /// Spawns one agent per available LLM provider.  Planned task items are
    /// distributed in round-robin order across providers and automatically
    /// re-routed to healthy providers on failure.  Requires the `col` feature.
    #[cfg(feature = "col")]
    #[arg(long, default_value_t = false)]
    pub collab: bool,

    /// Optional workspace path (can be "." for current directory)
    #[arg(value_name = "WORKSPACE")]
    pub workspace: Option<String>,

    /// Subcommands for autogpt.
    #[clap(subcommand)]
    pub command: Option<Commands>,
}

/// Represents available subcommands for the autogpt CLI.
#[cfg(feature = "cli")]
#[derive(Subcommand, Debug, PartialEq)]
pub enum Commands {
    /// Scaffold a new agent project from scratch
    #[command(
        name = "new",
        about = "Create a new agent project",
        long_about = "Scaffold a new AutoGPT-compatible agent project with YAML config and Cargo setup."
    )]
    New {
        /// Name of the agent project (used as directory name)
        #[arg()]
        name: String,

        /// AI provider feature to activate (e.g. gem, oai, cld, hf, etc.)
        #[arg(short, long, value_name = "FEATURE")]
        feature: Option<String>,
    },

    /// Build the Rust code generated from agent.yaml
    #[command(
        name = "build",
        about = "Compile generated agent source code",
        long_about = "Parses `agent.yaml`, generates Rust code in `src/main.rs` (or a custom location), and compiles the project using Cargo."
    )]
    Build {
        /// Optional output path for generated Rust file (default: src/main.rs)
        #[arg(short, long, value_name = "FILE")]
        out: Option<String>,
    },

    /// Run the compiled agent executable
    #[command(
        name = "run",
        about = "Run the compiled agent",
        long_about = "Executes the agent with the specified AI feature. This assumes the project is built successfully."
    )]
    Run {
        /// AI provider feature to activate (e.g. gem, oai, cld, etc.)
        #[arg(short, long, value_name = "FEATURE")]
        feature: Option<String>,
    },

    /// Perform a dry-run to validate the YAML file
    #[command(
        name = "test",
        about = "Validate agent.yaml file",
        long_about = "Checks the structure and required fields in `agent.yaml` without generating or compiling code."
    )]
    Test,

    #[clap(
        name = "man",
        about = "ManagerGPT: Generate complete project requirements, specs, and task plans."
    )]
    Man,

    #[clap(
        name = "arch",
        about = "ArchitectGPT: Design system architecture and component structure."
    )]
    Arch,

    #[clap(
        name = "front",
        about = "FrontendGPT: Build front-end applications, UIs, and interactive flows."
    )]
    Front,

    #[clap(
        name = "back",
        about = "BackendGPT: Develop APIs, services, and server-side logic."
    )]
    Back,

    #[clap(
        name = "design",
        about = "DesignerGPT: Create UI mockups, wireframes, and visual assets."
    )]
    Design,

    #[clap(
        name = "mail",
        about = "MailerGPT: Automate email content generation and outreach flows."
    )]
    Mail,

    #[clap(
        name = "git",
        about = "GitGPT: Automate Git commit messages, summaries, and version control tasks."
    )]
    Git,

    #[clap(
        name = "opt",
        about = "OptimizerGPT: Specializes in refactoring monolithic codebases into clean, modular components."
    )]
    Opt,

    /// Manage MCP (Model Context Protocol) server registrations.
    ///
    /// Add, list, remove, and inspect MCP servers that provide external tools
    /// to the CLI, TUI, and code-defined agents.
    #[clap(
        name = "mcp",
        about = "Manage MCP server configurations and inspect available tools."
    )]
    #[cfg(all(feature = "cli", feature = "mcp"))]
    Mcp {
        /// MCP subcommand.
        #[command(subcommand)]
        subcommand: McpSubcommand,
    },
}

/// Subcommands available under `autogpt mcp`.
#[cfg(feature = "cli")]
#[derive(ClapSubcommand, Debug, PartialEq)]
pub enum McpSubcommand {
    /// Register a new MCP server.
    #[clap(name = "add", about = "Register a new MCP server")]
    Add(Box<McpAddArgs>),

    /// List all registered MCP servers and their live connection status.
    #[clap(name = "list", about = "List all configured MCP servers")]
    List,

    /// Remove a registered MCP server.
    #[clap(name = "remove", about = "Remove a registered MCP server")]
    Remove {
        /// Name of the server to remove.
        name: String,
    },

    /// Show detailed information and available tools for a specific MCP server.
    #[clap(name = "inspect", about = "Inspect an MCP server and list its tools")]
    Inspect {
        /// Name of the server to inspect.
        name: String,
    },

    /// Call a specific tool on a registered MCP server.
    #[clap(name = "call", about = "Call an MCP tool")]
    Call {
        /// Name of the server.
        server: String,
        /// Name of the tool.
        tool: String,
        /// Optional tool arguments as JSON (key=value or '{...}').
        #[clap(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

/// Arguments for the `mcp add` command.
#[cfg(feature = "cli")]
#[derive(clap::Args, Debug, PartialEq)]
pub struct McpAddArgs {
    /// Unique name for the server (used as the map key in settings.json).
    pub name: String,

    /// For stdio transport: path to the executable.
    /// For http/sse transport: the endpoint URL.
    #[arg(short, long = "command", alias = "url")]
    pub command_or_url: String,

    /// Extra arguments passed to the server executable (stdio only).
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,

    /// Transport type: stdio (default), http, or sse.
    #[arg(short, long, default_value = "stdio")]
    pub transport: String,

    /// Environment variables in KEY=VALUE format (repeatable).
    #[arg(short, long = "env")]
    pub env_pairs: Vec<String>,

    /// HTTP headers in "Key: Value" format (repeatable, http/sse only).
    #[arg(short = 'H', long = "header")]
    pub headers: Vec<String>,

    /// Connection timeout in milliseconds.
    #[arg(long, default_value_t = 30_000)]
    pub timeout: u64,

    /// Trust this server, bypass all tool-call confirmation prompts.
    #[arg(long, default_value_t = false)]
    pub trust: bool,

    /// Human-readable description shown in `mcp list` and `mcp inspect`.
    #[arg(long)]
    pub description: Option<String>,

    /// Comma-separated list of tool names to expose (allowlist).
    #[arg(long = "include-tools")]
    pub include_tools: Vec<String>,

    /// Comma-separated list of tool names to hide (blocklist).
    #[arg(long = "exclude-tools")]
    pub exclude_tools: Vec<String>,
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
