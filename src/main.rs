use anyhow::Result;

/// The main entry point of `autogpt`.
///
/// It parses command-line arguments using the `clap` crate, configures agents based on
/// the provided command-line options, and performs an operation using the specified subcommand.
#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "cli")]
    {
        use autogpt::agents::architect::ArchitectGPT;
        use autogpt::agents::backend::BackendGPT;
        use autogpt::agents::frontend::FrontendGPT;
        use autogpt::agents::git::GitGPT;
        use autogpt::agents::manager::ManagerGPT;
        use autogpt::common::utils::ask_to_run_command;
        use autogpt::common::utils::setup_logging;
        use autogpt::common::utils::Scope;
        use autogpt::common::utils::Tasks;
        use autogpt::traits::functions::Functions;

        use autogpt::cli::{Cli, Commands};
        use clap::Parser;
        use colored::*;
        use std::env::var;
        use tracing::{error, info, warn};

        setup_logging()?;
        let args: Cli = Cli::parse();
        info!(
            "{}",
            "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?".bright_green()
        );

        let mut git_agent = GitGPT::new("Commit all changes", "GitGPT");

        let objective = "Expertise at managing projects at scale";
        let position = "ManagerGPT";
        let language = "python";

        let workspace = var("AUTOGPT_WORKSPACE")
            .unwrap_or("workspace/".to_string())
            .to_owned();
        if args.command.is_none() {
            loop {
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read line");

                input = input.trim().to_string();

                if !input.is_empty() {
                    let mut manager = ManagerGPT::new(objective, position, &input, language);
                    info!(
                        "{}",
                        "[*] \"AGI\": 🫡 Roger! Executing your command..."
                            .bright_yellow()
                            .bold()
                    );

                    let _ = manager.execute(true, false, 3).await;
                    info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
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
                        let mut manager = ManagerGPT::new(objective, position, &input, language);
                        info!(
                            "{}",
                            "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                .bright_yellow()
                                .bold()
                        );

                        let _ = manager.execute(true, false, 3).await;
                        info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                    } else {
                        warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                            .bright_yellow()
                                            .bold());
                    }
                },
                Commands::Arch => {
                    let objective = "Expertise at managing projects at scale";
                    let position = "ArchitectGPT";

                    let mut architect_agent = ArchitectGPT::new(objective, position);

                    let workspace = workspace + "architect";
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
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
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

                            architect_agent
                                .execute(&mut tasks, true, false, 3)
                                .await
                                .unwrap();
                            info!(
                                "{}",
                                "[*] \"AGI\": Committing the new code to Git..."
                                    .green()
                                    .bold()
                            );
                            let _ = git_agent.execute(&mut tasks, true, false, 1).await;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());

                            if let Err(e) = ask_to_run_command(
                                architect_agent.get_agent().clone(),
                                language,
                                &workspace,
                            )
                            .await
                            {
                                error!(
                                    "{}",
                                    format!("[*] AGI Runtime Error: {}", e).bright_red().bold()
                                );
                                break;
                            }
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                            .bright_yellow()
                                            .bold());

                            if let Err(e) = ask_to_run_command(
                                architect_agent.get_agent().clone(),
                                language,
                                &workspace,
                            )
                            .await
                            {
                                error!(
                                    "{}",
                                    format!("[*] AGI Runtime Error: {}", e).bright_red().bold()
                                );
                                break;
                            }
                        }
                    }
                }
                Commands::Front => {
                    let objective = "Expertise lies in writing frontend code";
                    let position = "FrontendGPT";

                    let workspace = workspace + "frontend";
                    let mut frontend_agent = FrontendGPT::new(objective, position, language);
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
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );

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

                            frontend_agent
                                .execute(&mut tasks, true, false, 3)
                                .await
                                .unwrap();
                            info!(
                                "{}",
                                "[*] \"AGI\": Committing the new code to Git..."
                                    .green()
                                    .bold()
                            );
                            let _ = git_agent.execute(&mut tasks, true, false, 1).await;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());

                            if let Err(e) = ask_to_run_command(
                                frontend_agent.get_agent().clone(),
                                language,
                                &workspace,
                            )
                            .await
                            {
                                error!(
                                    "{}",
                                    format!("[*] AGI Runtime Error: {}", e).bright_red().bold()
                                );
                                break;
                            }
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                            .bright_yellow()
                                            .bold());
                            if let Err(e) = ask_to_run_command(
                                frontend_agent.get_agent().clone(),
                                language,
                                &workspace,
                            )
                            .await
                            {
                                error!(
                                    "{}",
                                    format!("[*] AGI Runtime Error: {}", e).bright_red().bold()
                                );
                                break;
                            }
                        }
                    }
                }
                Commands::Back => {
                    let objective =
                        "Expertise lies in writing backend code for web servers and databases";
                    let position = "BackendGPT";
                    let workspace = workspace + "backend";
                    let mut backend_gpt = BackendGPT::new(objective, position, language);

                    let mut tasks = Tasks {
                        description: Default::default(),
                        scope: Some(Scope {
                            crud: true,
                            auth: true,
                            external: true,
                        }),
                        urls: None,
                        frontend_code: None,
                        backend_code: None,
                        api_schema: None,
                    };
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
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
                            tasks.description = input.into();

                            backend_gpt
                                .execute(&mut tasks, true, false, 3)
                                .await
                                .unwrap();
                            info!(
                                "{}",
                                "[*] \"AGI\": Committing the new code to Git..."
                                    .green()
                                    .bold()
                            );
                            let _ = git_agent.execute(&mut tasks, true, false, 1).await;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                            if let Err(e) = ask_to_run_command(
                                backend_gpt.get_agent().clone(),
                                language,
                                &workspace,
                            )
                            .await
                            {
                                error!(
                                    "{}",
                                    format!("[*] AGI Runtime Error: {}", e).bright_red().bold()
                                );
                                break;
                            }
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                                    .bright_yellow()
                                                    .bold());
                            if let Err(e) = ask_to_run_command(
                                backend_gpt.get_agent().clone(),
                                language,
                                &workspace,
                            )
                            .await
                            {
                                error!(
                                    "{}",
                                    format!("[*] AGI Runtime Error: {}", e).bright_red().bold()
                                );
                                break;
                            }
                        }
                    }
                }
                Commands::Design => {}
                Commands::Mail => {}
                Commands::Git => {}
            };
        }
    }

    Ok(())
}
