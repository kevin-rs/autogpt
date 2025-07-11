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
use clap::builder::styling::{AnsiColor, Effects, Styles};

/// Defines custom styles for CLI output.
#[cfg(feature = "cli")]
fn styles() -> Styles {
    // Define styles for different CLI elements
    clap::builder::styling::Styles::styled()
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

The `autogpt` CLI enables interaction with the Orchestrator and/or an AI Provider
through a suite of built-in, specialized autonomous AI agents designed for various
stages of project development.

Modes of Operation:
-------------------
Autogpt supports 2 modes:

1. Networking (Agentic) Mode (default):
   When no subcommand is provided, `autogpt` runs as a networked agent that connects
   to an orchestrator (`orchgpt`) over TLS-encrypted TCP. The orchestrator can run
   on the same or a separate machine.

2. Networkless (Agentic) Mode:
   When a subcommand is specified, `autogpt` runs locally in standalone mode, without
   requiring a connection to an orchestrator.
"#

)]
pub struct Cli {
    #[clap(global = true, short, long)]
    pub verbose: bool,

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
}
