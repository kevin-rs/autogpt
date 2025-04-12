#[cfg(feature = "cli")]
use clap::Parser;

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

/// Represents the command-line interface (CLI) for orchgpt.
#[cfg(feature = "cli")]
#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    name = "orchgpt",
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
 ██████  ██████   ██████ ██   ██  ██████  ██████  ████████ 
██    ██ ██   ██ ██      ██   ██ ██       ██   ██    ██    
██    ██ ██████  ██      ███████ ██   ███ ██████     ██    
██    ██ ██   ██ ██      ██   ██ ██    ██ ██         ██    
 ██████  ██   ██  ██████ ██   ██  ██████  ██         ██    

The `orchgpt` CLI is the central orchestrator that manages communication and execution
of autonomous agents in the AutoGPT ecosystem.

It acts as the control plane in **networking mode**, receiving commands from one or more
`autogpt` CLI instances over TLS-encrypted TCP. Based on incoming instructions, the
orchestrator is responsible for creating, running, routing to, or terminating agent instances.

Modes of Operation:
-------------------
This CLI is only used in **Networking Mode**. It must be running before any `autogpt` 
instance attempts to connect without a subcommand.

Responsibilities:
-----------------
- Parses incoming commands from `autogpt`.
- Spawns appropriate agent processes based on the request.
- Manages agent lifecycle (creation, routing, termination).
- Facilitates secure, scalable multi-agent collaboration.
"#
)]
pub struct Cli {
    #[clap(global = true, short, long)]
    pub verbose: bool,
}
