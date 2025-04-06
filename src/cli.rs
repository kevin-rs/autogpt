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
{about-with-newline}

{usage-heading} {usage}

{all-args}{after-help}

AUTHORS:
    {author}
"#,
    about=r#"

Autogpt CLI
===========

Autogpt cli allows you to chat with the gemini api using built-in specialized autonomous AI agents.

SUBCOMMANDS:
  - man: ManagerGPT agent for generating project requirements.
  - arch: ArchitectGPT agent for architecture design.
  - front: FrontendGPT agent for front-end development.
  - back: BackendGPT agent for back-end development.
  - design: DesignerGPT agent for graphic design.
  - mail: MailerGPT agent for email automation.

USAGE:
  autogpt [SUBCOMMAND]

EXAMPLES:
  1. Generate an entire project in one command:
      autogpt

  2. Generate a design architecture for a project:
      autogpt arch

  3. Develop a frontend app:
      autogpt front

  4. Develop backend app:
      autogpt back

  5. Design UIs and Wireframes:
      autogpt design

  6. Automate emails:
      autogpt mail

  7. Automate Git Commit hell:
      autogpt git

  8. Get help with a specific subcommand:
      autogpt man --help
      autogpt arch --help
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
#[derive(Subcommand, Debug)]
pub enum Commands {
    #[clap(
        name = "man",
        about = "Manage AI agent for generating an entire project."
    )]
    Man,

    #[clap(name = "arch", about = "Manage AI agent for architecture design.")]
    Arch,

    #[clap(name = "front", about = "Manage AI agent for front-end development.")]
    Front,

    #[clap(name = "back", about = "Manage AI agent for back-end development.")]
    Back,

    #[clap(name = "design", about = "Manage AI agent for graphic design.")]
    Design,

    #[clap(name = "mail", about = "Manage AI agent for email automation.")]
    Mail,

    #[clap(name = "git", about = "Manage AI agent for git commit automation.")]
    Git,
}
