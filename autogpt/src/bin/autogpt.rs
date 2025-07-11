use anyhow::Result;

/// The main entry point of `autogpt`.
///
/// This function parses command-line arguments using the `clap` crate, sets up agent configurations
/// based on the provided options, and executes operations according to the specified subcommand.
///
/// `autogpt` supports two modes of operation:
///
/// 1. **Networking Mode**: In this mode, `autogpt` acts as a networked agent that communicates
///    with an orchestrator over TLS-encrypted TCP. The orchestrator must be running on a machine
///    via the `orchgpt` command. `autogpt` can then connect to it either from the same machine or
///    from another machine.
///
/// 2. **Networkless (Agentic) Mode**: In this standalone mode, `autogpt` operates independently
///    without requiring a network connection to an orchestrator. It runs locally, executing tasks
///    based solely on local configurations and input.
//
/// This flexible design allows `autogpt` to be deployed in a distributed multi-agent environment
/// or as a single self-contained agents.
#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "cli")]
    {
        use autogpt::agents::architect::ArchitectGPT;
        use autogpt::agents::backend::BackendGPT;
        use autogpt::agents::designer::DesignerGPT;
        use autogpt::agents::frontend::FrontendGPT;
        use autogpt::agents::git::GitGPT;
        use autogpt::agents::mailer::MailerGPT;
        use autogpt::agents::manager::ManagerGPT;
        use autogpt::agents::optimizer::OptimizerGPT;
        use autogpt::cli::autogpt::commands::{build, new, run, test};
        use autogpt::cli::autogpt::{Cli, Commands};

        use autogpt::common::input::read_user_input;
        use autogpt::common::utils::Scope;
        use autogpt::common::utils::Task;
        use autogpt::common::utils::ask_to_run_command;
        use autogpt::common::utils::setup_logging;
        use autogpt::traits::functions::AsyncFunctions;
        use autogpt::traits::functions::Functions;
        use clap::Parser;
        use std::env::var;

        use autogpt::common::tls::load_certs;
        use autogpt::message::Message;
        use autogpt::message::encode_message;
        use colored::*;
        use rustls::pki_types::ServerName;
        use rustls::{ClientConfig, RootCertStore};
        use std::env;
        use std::io::Write;
        use std::sync::Arc;
        use tokio::io::AsyncBufReadExt;
        use tokio::io::AsyncReadExt;
        use tokio::io::AsyncWriteExt;
        use tokio::net::TcpStream;
        use tokio::signal;
        use tokio::time::Duration;
        use tokio::time::timeout;
        use tokio_rustls::TlsConnector;
        use tracing::{error, info, warn};

        setup_logging()?;

        let args: Cli = Cli::parse();

        async fn run_client() -> Result<()> {
            let certs = load_certs("certs/cert.pem")?;
            let mut root = RootCertStore::empty();
            root.add_parsable_certificates(certs);

            let config = ClientConfig::builder()
                .with_root_certificates(root)
                .with_no_client_auth();

            let connector = TlsConnector::from(Arc::new(config));
            let bind_address =
                env::var("ORCHESTRATOR_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8443".to_string());

            let stream = TcpStream::connect(&bind_address).await?;
            let domain = ServerName::try_from("localhost")?;
            let mut tls_stream = connector.connect(domain, stream).await?;

            let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
            let mut input_line = String::new();

            let shutdown_signal = signal::ctrl_c();

            tokio::pin!(shutdown_signal);

            loop {
                print!("> ");
                std::io::stdout().flush()?;

                input_line.clear();
                tokio::select! {
                    read = stdin.read_line(&mut input_line) => {
                        if read? == 0 {
                            break;
                        }

                        let input = input_line.trim().to_string();
                        if input.is_empty() {
                            warn!(
                                "{}",
                                "[*] \"AGI\": 🤔 You've entered an empty command? What exactly does that entail?"
                                    .bright_yellow()
                                    .bold()
                            );
                            continue;
                        }

                        if !input.starts_with('/') {
                            error!(
                                "{}",
                                "[*] \"AGI\": ❌ Command must begin with a '/' followed by the agent name."
                                    .bright_red()
                                    .bold()
                            );
                            continue;
                        }

                        if let Some((to, rest)) = input.split_once(' ') {
                            let mut parts = rest.trim().splitn(2, ' ');
                            if let Some(action) = parts.next() {
                                let (actual_input, lang) = if action.eq_ignore_ascii_case("create") {
                                    ("", "")
                                } else if let Some(remaining) = parts.next() {
                                    if let Some((input, lang)) = remaining.split_once('|') {
                                        (input.trim(), lang.trim())
                                    } else {
                                        (remaining.trim(), "python")
                                    }
                                } else {
                                    error!(
                                        "{}",
                                        "[*] \"AGI\": ❌ Invalid command format. Use: /<agent> <action>"
                                            .bright_red()
                                            .bold()
                                    );
                                    continue;
                                };

                                let payload = format!("input={actual_input};language={lang}");

                                let msg = Message {
                                    from: "cli".to_string(),
                                    to: to.trim_start_matches('/').to_string(),
                                    msg_type: action.into(),
                                    payload_json: payload,
                                    auth_token: "secret".into(),
                                };

                                let data = encode_message(&msg)?;
                                tls_stream.write_all(&data).await?;
                                info!("{}", "[*] \"AGI\": ✅ Sent!".green().bold());

                                let mut buf = vec![0u8; 4096];
                                let n = match timeout(Duration::from_secs(30), tls_stream.read(&mut buf)).await {
                                    Ok(Ok(n)) if n > 0 => n,
                                    Ok(_) => {
                                        warn!("{}", "[*] \"AGI\": ❌ Connection closed.".bright_red().bold());
                                        break;
                                    },
                                    Err(_) => {
                                        warn!("{}", "[*] \"AGI\": ⏱️ Timeout waiting for response.".bright_red().bold());
                                        break;
                                    }
                                };

                                let response = String::from_utf8_lossy(&buf[..n]);
                                info!(
                                    "{} {}",
                                    "[*] \"AGI\": 📬 Got Response →".bright_green().bold(),
                                    response.trim()
                                );
                            }
                        }
                    }
                    _ = &mut shutdown_signal => {
                        info!("\n[*] \"AGI\": 👋 Graceful shutdown requested.");
                        let _ = tls_stream.shutdown().await;
                        std::process::exit(0);
                    }
                }
            }

            Ok(())
        }

        if args.command.is_none() {
            // If no command specified, default to networking mode (connects to an orchestrator).
            info!(
                "{}",
                "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?".bright_green()
            );
            loop {
                match run_client().await {
                    Ok(_) => {
                        break;
                    }
                    Err(e) => {
                        error!("Client error: {}", e);
                        info!("{}", "[*] \"AGI\": 🔁 Reconnecting in 3s...".bright_green());
                        tokio::time::sleep(Duration::from_secs(3)).await;
                    }
                }
            }
        } else if let Some(command) = args.command {
            // If a command is provided, operate in networkless (standalone agents) mode.
            let mut git_agent = GitGPT::default();
            let mut _optimizer_gpt = OptimizerGPT::default();
            let language = "python";
            let workspace = var("AUTOGPT_WORKSPACE")
                .unwrap_or("workspace/".to_string())
                .to_owned();
            if !matches!(
                command,
                Commands::Test
                    | Commands::Run { feature: _ }
                    | Commands::Build { out: _ }
                    | Commands::New { name: _ }
            ) {
                git_agent = GitGPT::new("Commit all changes", "GitGPT").await;

                let objective =
                    "Expertise lies in modularizing monolithic source code into clean components";
                let position = "OptimizerGPT";

                _optimizer_gpt = OptimizerGPT::new(objective, position, language).await;
            }
            match command {
                Commands::Man => {
                    let objective = "Expertise at managing projects at scale";
                    let position = "ManagerGPT";
                    #[allow(unused_assignments)]
                    let mut manager = ManagerGPT::new(objective, position, "", language);

                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        let input = read_user_input()?;
                        manager = ManagerGPT::new(objective, position, &input, language);

                        if !input.is_empty() {
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );

                            let _ = manager.execute(true, true, 3).await;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                                .bright_yellow()
                                                .bold());
                        }
                    }
                }
                Commands::Arch => {
                    let objective = "Expertise at managing projects at scale";
                    let position = "ArchitectGPT";

                    let mut architect_agent = ArchitectGPT::new(objective, position).await;

                    let workspace = workspace + "architect";
                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        let input = read_user_input()?;

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
                            let mut tasks = Task {
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
                                    format!("[*] AGI Runtime Error: {e}").bright_red().bold()
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
                                    format!("[*] AGI Runtime Error: {e}").bright_red().bold()
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
                    let mut frontend_agent = FrontendGPT::new(objective, position, language).await;

                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        let input = read_user_input()?;

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );

                            let mut tasks = Task {
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
                                    format!("[*] AGI Runtime Error: {e}").bright_red().bold()
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
                                    format!("[*] AGI Runtime Error: {e}").bright_red().bold()
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
                    let mut backend_gpt = BackendGPT::new(objective, position, language).await;

                    let mut tasks = Task {
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

                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        let input = read_user_input()?;

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
                                    format!("[*] AGI Runtime Error: {e}").bright_red().bold()
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
                                    format!("[*] AGI Runtime Error: {e}").bright_red().bold()
                                );
                                break;
                            }
                        }
                    }
                }
                Commands::Design => {
                    let objective = "Crafts stunning web design layouts";
                    let position = "Web Designer";
                    let mut designer_agent = DesignerGPT::new(objective, position).await;

                    let mut tasks = Task {
                        description: "".into(),
                        scope: None,
                        urls: None,
                        backend_code: None,
                        frontend_code: None,
                        api_schema: None,
                    };

                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        let input = read_user_input()?;

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
                            tasks.description = input.into();

                            designer_agent.execute(&mut tasks, true, false, 3).await?;

                            info!(
                                "{}",
                                "[*] \"AGI\": Committing the new files to Git..."
                                    .green()
                                    .bold()
                            );

                            git_agent.execute(&mut tasks, true, false, 1).await?;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                                    .bright_yellow()
                                                    .bold());
                        }
                    }
                }
                Commands::Mail => {
                    let objective = "Expertise at summarizing emails";
                    let position = "Mailer";

                    let mut mailer_agent = MailerGPT::new(objective, position).await;
                    let mut tasks = Task {
                        description: "".into(),
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

                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        let input = read_user_input()?;

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
                            tasks.description = input.into();

                            let _ = mailer_agent.execute(&mut tasks, true, false, 3).await;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                                    .bright_yellow()
                                                    .bold());
                        }
                    }
                }
                Commands::Git => {
                    let objective = "Commit all changes";
                    let position = "GitGPT";

                    let mut git_agent = GitGPT::new(objective, position).await;
                    let mut tasks = Task {
                        description: "".into(),
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

                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        let input = read_user_input()?;

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
                            tasks.description = input.into();

                            let _result = git_agent.execute(&mut tasks, true, false, 1).await;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                                    .bright_yellow()
                                                    .bold());
                        }
                    }
                }
                Commands::Opt => {
                    let objective = "Optimize and modularize backend code";
                    let position = "OptimizerGPT";

                    let mut optimizer_agent =
                        OptimizerGPT::new(objective, position, language).await;

                    let mut tasks = Task {
                        description: "".into(),
                        scope: None,
                        urls: None,
                        frontend_code: None,
                        backend_code: None,
                        api_schema: None,
                    };

                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        let input = read_user_input()?;

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
                            tasks.description = input.into();

                            let _result = optimizer_agent.execute(&mut tasks, true, false, 1).await;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                                    .bright_yellow()
                                                    .bold());
                        }
                    }
                }
                Commands::New { name } => new::handle_new(&name)?,
                Commands::Build { out } => build::handle_build(out)?,
                Commands::Run { feature } => run::handle_run(feature)?,
                Commands::Test => test::handle_test()?,
            };
        }
    }
    Ok(())
}
