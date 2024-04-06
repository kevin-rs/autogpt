use anyhow::Result;

/// The main entry point of `autogpt`.
///
/// It parses command-line arguments using the `clap` crate, configures agents based on
/// the provided command-line options, and performs an operation using the specified subcommand.
#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "cli")]
    {
        use tracing_subscriber::{filter, fmt};

        let filter = filter::LevelFilter::INFO;
        use autogpt::agents::architect::ArchitectGPT;
        use autogpt::agents::frontend::FrontendGPT;
        use autogpt::agents::manager::ManagerGPT;
        use autogpt::common::utils::Scope;
        use autogpt::common::utils::Tasks;
        use autogpt::traits::functions::Functions;

        use autogpt::cli::{Cli, Commands};
        use clap::Parser;
        use colored::*;
        use tracing::{info, warn};

        // Start configuring a `fmt` subscriber
        let subscriber = fmt()
            .compact()
            .with_max_level(filter)
            .with_file(false)
            .with_line_number(false)
            .with_thread_ids(false)
            .with_target(false)
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
        let args: Cli = Cli::parse();
        info!(
            "{}",
            "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?".bright_green()
        );

        if args.command.is_none() {
            loop {
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read line");

                input = input.trim().to_string();

                if !input.is_empty() {
                    let input = input.clone();
                    info!(
                        "{}",
                        "[*] \"AGI\": 🚀 Roger! Executing your command..."
                            .bright_yellow()
                            .bold()
                    );

                    let objective = "Expertise at managing projects at scale";
                    let position = "ManagerGPT";
                    let language = "python";

                    let mut manager = ManagerGPT::new(objective, position, &input, language);

                    let _ = manager.execute(true, 3).await;
                    info!("{}", "[*] \"AGI\": ✅ Done! What would you like to build next? Perhaps you'd like to improve the app?".green().bold());
                } else {
                    warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                            .bright_yellow()
                                            .bold());
                }
            }
        } else if let Some(command) = args.command {
            match command {
                Commands::Man => loop {
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to read line");

                    input = input.trim().to_string();

                    if !input.is_empty() {
                        let input = input.clone();
                        info!(
                            "{}",
                            "[*] \"AGI\": 🚀 Roger! Executing your command..."
                                .bright_yellow()
                                .bold()
                        );

                        let objective = "Expertise at managing projects at scale";
                        let position = "ManagerGPT";
                        let language = "python";

                        let mut manager = ManagerGPT::new(objective, position, &input, language);

                        let _ = manager.execute(true, 3).await;
                        info!("{}", "[*] \"AGI\": ✅ Done! What would you like to build next? Perhaps you'd like to improve the app?".green().bold());
                    } else {
                        warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                            .bright_yellow()
                                            .bold());
                    }
                },
                Commands::Arch => loop {
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to read line");

                    input = input.trim().to_string();

                    if !input.is_empty() {
                        let input = input.clone();
                        info!(
                            "{}",
                            "[*] \"AGI\": 🚀 Roger! Executing your command..."
                                .bright_yellow()
                                .bold()
                        );

                        let objective = "Expertise at managing projects at scale";
                        let position = "ArchitectGPT";

                        let mut architect_agent = ArchitectGPT::new(objective, position);

                        let mut tasks = Tasks {
                            description: input.into(),
                            scope: Some(Scope {
                                crud: true,
                                auth: false,
                                external: true,
                            }),
                            urls: None,
                            frontend_code: None,
                            backend_code: None,
                            api_schema: None,
                        };

                        architect_agent.execute(&mut tasks, true, 3).await.unwrap();
                        info!("{}", "[*] \"AGI\": ✅ Done! What would you like to build next? Perhaps you'd like to improve the app?".green().bold());
                    } else {
                        warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                            .bright_yellow()
                                            .bold());
                    }
                },
                Commands::Front => loop {
                    let mut input = String::new();
                    std::io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to read line");

                    input = input.trim().to_string();

                    if !input.is_empty() {
                        let input = input.clone();
                        info!(
                            "{}",
                            "[*] \"AGI\": 🚀 Roger! Executing your command..."
                                .bright_yellow()
                                .bold()
                        );

                        let objective = "Expertise lies in writing frontend code";
                        let position = "FrontendGPT";
                        let language = "python";

                        let mut frontend_agent = FrontendGPT::new(objective, position, language);

                        let mut tasks = Tasks {
                            description: input.into(),
                            scope: Some(Scope {
                                crud: true,
                                auth: false,
                                external: true,
                            }),
                            urls: None,
                            frontend_code: None,
                            backend_code: None,
                            api_schema: None,
                        };

                        frontend_agent.execute(&mut tasks, true, 3).await.unwrap();
                        info!("{}", "[*] \"AGI\": ✅ Done! What would you like to build next? Perhaps you'd like to improve the app?".green().bold());
                    } else {
                        warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                            .bright_yellow()
                                            .bold());
                    }
                },
                Commands::Back => {}
                Commands::Design => {}
                Commands::Mail => {}
            };
        }
    }

    Ok(())
}
