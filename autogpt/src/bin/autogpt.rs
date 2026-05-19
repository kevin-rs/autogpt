// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use anyhow::Result;
use std::env::{current_dir, set_var, var};
use std::fs::read_to_string;
#[cfg(all(
    feature = "cli",
    any(
        feature = "gem",
        feature = "oai",
        feature = "cld",
        feature = "xai",
        feature = "co",
        feature = "hf",
        feature = "net"
    )
))]
use std::io;

#[cfg(feature = "mcp")]
use dirs::home_dir;
#[cfg(all(
    feature = "cli",
    any(
        feature = "gem",
        feature = "oai",
        feature = "cld",
        feature = "xai",
        feature = "co",
        feature = "hf",
        feature = "net"
    )
))]
use std::io::Write;

/// The main entry point of `autogpt`.
///
/// This function parses command-line arguments using the `clap` crate, sets up agent configurations
/// based on the provided options, and executes operations according to the specified subcommand.
///
/// `autogpt` supports four modes of operation:
///
/// 1. **Interactive Mode (Default)**: Launches an interactive AI shell powered by GenericGPT
///    when run with no subcommands or flags. It features session persistence, model switching,
///    and multi-provider support.
///
/// 2. **Direct Prompt Mode**: Uses the `-p` flag to interact with the LLM directly without
///    configuring agents. Can be combined with `--mixture` for multi-provider responses.
///
/// 3. **Agentic Networkless Mode (Standalone)**: Runs individual specialized agents directly
///    via subcommands (e.g., `arch`, `back`, `front`). Each agent operates independently
///    without a networked orchestrator.
///
/// 4. **Agentic Networking Mode (Orchestrated)**: Connects to an external orchestrator (`orchgpt`)
///    over a secure TLS-encrypted TCP channel using the IAC protocol for distributed collaboration.
///
/// This flexible design allows `autogpt` to be deployed in a distributed multi-agent environment
/// or as a single self-contained agent.
fn load_dotenv() {
    let mut paths = Vec::new();
    if let Ok(cwd) = current_dir() {
        paths.push(cwd.join(".env"));
    }
    #[cfg(feature = "mcp")]
    {
        if let Some(home) = home_dir() {
            paths.push(home.join(".env"));
            paths.push(home.join(".autogpt").join(".env"));
        }
    }

    for path in paths {
        if path.exists()
            && let Ok(content) = read_to_string(&path)
        {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    let k = k.trim();
                    let v = v.trim().trim_matches('"').trim_matches('\'');
                    if !k.is_empty() && var(k).is_err() {
                        unsafe {
                            set_var(k, v);
                        }
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    load_dotenv();
    #[cfg(feature = "cli")]
    {
        #[allow(unused_imports)]
        use {
            anyhow::anyhow,
            clap::Parser,
            colored::Colorize,
            std::env,
            termimad::MadSkin,
            tracing::{error, info, warn},
        };

        use autogpt::{
            agents::generic::{GenericAgentLoopConfig, run_generic_agent_loop},
            cli::autogpt::{
                Cli, Commands,
                commands::{build, new, run, test},
            },
            cli::settings::SettingsManager,
            common::utils::{fetch_latest_version, is_outdated, prompt_for_update, setup_logging},
            prelude::ClientType,
            tui::{app::TuiApp, state::TuiEvent},
        };

        #[cfg(feature = "gpt")]
        use {
            autogpt::{
                agents::{
                    architect::ArchitectGPT, backend::BackendGPT, designer::DesignerGPT,
                    frontend::FrontendGPT, manager::ManagerGPT, optimizer::OptimizerGPT,
                },
                common::{
                    input::read_user_input,
                    utils::{Scope, Task, ask_to_run_command},
                },
                traits::functions::{AsyncFunctions, Functions},
            },
            std::env::var,
        };

        #[cfg(all(feature = "gpt", feature = "git"))]
        use autogpt::agents::git::GitGPT;

        #[cfg(all(feature = "gpt", feature = "mail"))]
        use autogpt::agents::mailer::MailerGPT;

        #[cfg(feature = "mop")]
        use autogpt::agents::mop::run_mixture;

        #[cfg(feature = "gem")]
        use {
            autogpt::prelude::CTrait,
            gems::{
                messages::{Content, Message as GemMessage},
                models::Model,
                stream::StreamBuilder,
                utils::extract_text_from_partial_json,
            },
        };

        #[cfg(any(feature = "gem", feature = "oai", feature = "cld"))]
        use futures_util::StreamExt;

        #[cfg(feature = "net")]
        use {
            iac_rs::{message::Message, prelude::*},
            std::sync::Arc,
            tokio::{io::AsyncBufReadExt, signal, sync::Mutex, time::timeout},
        };

        #[cfg(any(
            feature = "gem",
            feature = "oai",
            feature = "cld",
            feature = "xai",
            feature = "co",
            feature = "hf",
            feature = "net"
        ))]
        use {std::thread, tokio::time::Duration};

        #[cfg(all(feature = "cli", feature = "mcp"))]
        use {
            autogpt::cli::autogpt::McpSubcommand, autogpt::cli::autogpt::commands::mcp as mcp_cmd,
        };
        let args: Cli = Cli::parse();

        let current_version = env!("CARGO_PKG_VERSION");

        let tui_mode = args.prompt.is_none() && args.command.is_none() && !args.net;
        setup_logging(tui_mode)?;
        let mut update_available = None;
        if let Some(latest_version) = fetch_latest_version().await
            && is_outdated(current_version, &latest_version)
        {
            if tui_mode {
                update_available = Some((current_version.to_string(), latest_version));
            } else {
                prompt_for_update();
            }
        }

        #[allow(dead_code)]
        #[cfg(any(
            feature = "gem",
            feature = "oai",
            feature = "cld",
            feature = "xai",
            feature = "co",
            feature = "hf",
            feature = "net"
        ))]
        pub fn type_with_cursor_effect(text: &str, delay: u64, skin: &MadSkin) {
            skin.print_inline(text);
            let _ = io::stdout().flush();
            thread::sleep(Duration::from_millis(delay));
        }

        #[cfg(feature = "net")]
        async fn run_client() -> Result<()> {
            let signer = Signer::new(KeyPair::generate());
            let address =
                env::var("ORCHESTRATOR_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8443".to_string());
            let client = Arc::new(Mutex::<Client>::new(
                Client::connect(&address, signer.clone()).await?,
            ));
            let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
            let mut input_line = String::new();
            let public_key_bytes = signer.verifying_key().as_slice().to_vec();
            let register_key_msg = Message {
                from: "autogpt".into(),
                to: "orchestrator".into(),
                msg_type: MessageType::RegisterKey,
                extra_data: public_key_bytes,
                ..Default::default()
            };
            client.lock().await.send(register_key_msg).await?;
            let shutdown_signal = signal::ctrl_c();
            tokio::pin!(shutdown_signal);
            loop {
                print!("> ");
                io::stdout().flush()?;
                input_line.clear();
                tokio::select! {
                    read = stdin.read_line(&mut input_line) => {
                        if read? == 0 {
                            break;
                        }
                        let input = input_line.trim();
                        if input.is_empty() {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty command?".bright_yellow().bold());
                            continue;
                        }

                        if !input.starts_with('/') {
                            error!("{}", "[*] \"AGI\": ❌ Command must begin with a '/' followed by the agent name.".bright_red().bold());
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
                                    error!("{}", "[*] \"AGI\": ❌ Invalid command format. Use: /<agent> <action>".bright_red().bold());
                                    continue;
                                };

                                let payload = format!("input={actual_input};language={lang}");

                                let msg = Message {
                                    from: "cli".into(),
                                    to: to.trim_start_matches('/').into(),
                                    msg_type: action.into(),
                                    payload_json: payload,
                                    ..Default::default()
                                };

                                let client = client.lock().await;
                                client.send(msg).await?;
                                info!("{}", "[*] \"AGI\": ✅ Sent!".green().bold());

                                let response = timeout(Duration::from_secs(30), client.receive()).await?;

                                match response {
                                    Ok(Some(resp)) => {
                                        info!(
                                            "{} {}",
                                            "[*] \"AGI\": 📬 Got Response →".bright_green().bold(),
                                            resp.payload_json.trim()
                                        );
                                    }
                                    Ok(None) => {
                                        warn!("{}", "[*] \"AGI\": ❌ No response from server.".bright_red().bold());
                                        break;
                                    }
                                    Err(_) => {
                                        warn!("{}", "[*] \"AGI\": ⏱️ Timeout waiting for response.".bright_red().bold());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    _ = &mut shutdown_signal => {
                        info!("\n[*] \"AGI\": 👋 Graceful shutdown requested.");
                        break;
                    }
                }
            }

            Ok(())
        }
        if let Some(_prompt) = args.prompt {
            #[allow(unused_variables)]
            let prompt = _prompt;
            #[cfg(feature = "mop")]
            if args.mixture {
                use tracing::info;
                if let Some((provider, response)) = run_mixture(&prompt).await {
                    info!(
                        "{}",
                        format!("[*] MoP: selected response from {}", provider)
                            .bright_green()
                            .bold()
                    );
                    let skin = MadSkin::default();
                    skin.print_text(&response);
                    return Ok(());
                }
            }

            #[allow(unused_variables)]
            let skin = MadSkin::default();
            let mut client = ClientType::from_env();
            match &mut client {
                #[cfg(feature = "gem")]
                ClientType::Gemini(gem_client) => {
                    let parameters = StreamBuilder::default()
                        .model(Model::Flash3Preview)
                        .input(GemMessage::User {
                            content: Content::Text(prompt),
                            name: None,
                        })
                        .build()?;

                    let response = gem_client.stream().generate(parameters).await?;
                    let mut stream = response.bytes_stream();

                    let delay = 1;
                    let mut message: String = Default::default();
                    while let Some(mut chunk) = stream.next().await {
                        if let Ok(parsed_json) = std::str::from_utf8(chunk.as_mut().unwrap()) {
                            if let Some(text_value) = extract_text_from_partial_json(parsed_json) {
                                let lines: Vec<&str> = text_value
                                    .split("\\n")
                                    .flat_map(|s| s.split('\n'))
                                    .collect();
                                for line in lines {
                                    message.push_str(&line.replace('\\', ""));
                                    if !line.is_empty() {
                                        type_with_cursor_effect(
                                            &line.replace('\\', ""),
                                            delay,
                                            &skin,
                                        );
                                    } else {
                                        println!("\n");
                                    }
                                }
                            }
                        } else {
                            error!("Failed to parse chunk: {:?}", chunk.as_ref().unwrap());
                        }
                    }
                }

                #[cfg(feature = "oai")]
                ClientType::OpenAI(oai_client) => {
                    use openai_dive::v1::resources::chat::{
                        ChatCompletionParametersBuilder, ChatMessage, ChatMessageContent,
                    };

                    let parameters = ChatCompletionParametersBuilder::default()
                        .model("gpt-5")
                        .messages(vec![ChatMessage::User {
                            content: ChatMessageContent::Text(prompt),
                            name: None,
                        }])
                        .build()?;

                    let mut stream = oai_client.chat().create_stream(parameters).await?;

                    while let Some(response) = stream.next().await {
                        match response {
                            Ok(chat_response) => {
                                chat_response.choices.iter().for_each(|choice| {
                                    match &choice.delta {
                                        openai_dive::v1::resources::chat::DeltaChatMessage::Assistant {
                                            content: Some(openai_dive::v1::resources::chat::ChatMessageContent::Text(text)),
                                            ..
                                        } => type_with_cursor_effect(text, 1, &skin),
                                        openai_dive::v1::resources::chat::DeltaChatMessage::Untagged {
                                            content: Some(openai_dive::v1::resources::chat::ChatMessageContent::Text(text)),
                                            ..
                                        } => type_with_cursor_effect(text, 1, &skin),
                                        _ => {}
                                    }
                                });
                            }
                            Err(e) => error!("OpenAI chat streaming failed: {}", e),
                        }
                    }
                }

                #[cfg(feature = "cld")]
                ClientType::Anthropic(client) => {
                    use anthropic_ai_sdk::types::message::MessageClient;
                    use anthropic_ai_sdk::types::message::{
                        CreateMessageParams, Message, RequiredMessageParams, Role,
                    };

                    let body = CreateMessageParams::new(RequiredMessageParams {
                        model: "claude-opus-4-6".to_string(),
                        messages: vec![Message::new_text(Role::User, prompt.to_string())],
                        max_tokens: 1024,
                    })
                    .with_stream(true);

                    match client.create_message_streaming(&body).await {
                        Ok(mut stream) => {
                            while let Some(event_result) = stream.next().await {
                                match event_result {
                                    Ok(anthropic_ai_sdk::types::message::StreamEvent::ContentBlockDelta { delta, .. }) => {
                                        if let anthropic_ai_sdk::types::message::ContentBlockDelta::TextDelta { text } = delta {
                                            type_with_cursor_effect(&text, 1, &skin);
                                        }
                                    }
                                    Ok(_) => {}
                                    Err(e) => error!("Anthropic chat streaming failed: {}", e),
                                }
                            }
                        }
                        Err(e) => error!("Anthropic API failed to initiate stream: {}", e),
                    }
                }

                #[cfg(feature = "xai")]
                ClientType::Xai(xai_client) => {
                    use x_ai::chat_compl::{ChatCompletionsRequestBuilder, Message as XaiMessage};
                    use x_ai::traits::ClientConfig;

                    let messages = vec![XaiMessage::text("user", prompt.to_string())];
                    let req = ChatCompletionsRequestBuilder::new(
                        xai_client.clone(),
                        "grok-4".into(),
                        messages,
                    )
                    .stream(true)
                    .build()?;

                    let response_stream_res = xai_client
                        .request(reqwest::Method::POST, "chat/completions")
                        .map_err(|e| anyhow!("Failed to build xAI request: {}", e))?
                        .json(&req)
                        .send()
                        .await;

                    match response_stream_res {
                        Ok(mut resp) => {
                            let mut buffer = String::new();

                            while let Ok(Some(chunk)) = resp.chunk().await {
                                if let Ok(text) = std::str::from_utf8(&chunk) {
                                    buffer.push_str(text);
                                    let mut parts: Vec<&str> = buffer.split("\n\n").collect();
                                    let new_buffer = if !buffer.ends_with("\n\n") {
                                        parts.pop().unwrap_or("").to_string()
                                    } else {
                                        String::new()
                                    };

                                    for part in parts {
                                        for line in part.lines() {
                                            if line.starts_with("data: ") {
                                                let data = line.trim_start_matches("data: ").trim();
                                                if data == "[DONE]" {
                                                    continue;
                                                }
                                                if let Some(content) =
                                                    serde_json::from_str::<serde_json::Value>(data)
                                                        .ok()
                                                        .and_then(|json| {
                                                            json.get("choices")
                                                                .and_then(|c| c.as_array())
                                                                .and_then(|c| c.first())
                                                                .and_then(|c| c.get("delta"))
                                                                .and_then(|d| d.get("content"))
                                                                .and_then(|c| c.as_str())
                                                                .map(|s| s.to_string())
                                                        })
                                                {
                                                    type_with_cursor_effect(&content, 1, &skin);
                                                }
                                            }
                                        }
                                    }
                                    buffer = new_buffer;
                                }
                            }
                        }
                        Err(e) => error!("xAI API request failed: {}", e),
                    }
                }

                #[cfg(feature = "co")]
                ClientType::Cohere(co_client) => {
                    use cohere_rust::api::GenerateModel;
                    use cohere_rust::api::chat::ChatRequest;

                    let chat_request = ChatRequest {
                        message: &prompt,
                        model: Some(GenerateModel::Custom("command-a-03-2025".to_string())),
                        max_tokens: Some(4096),
                        ..Default::default()
                    };

                    match co_client.chat(&chat_request).await {
                        Ok(mut receiver) => {
                            while let Some(res) = receiver.recv().await {
                                if let Ok(cohere_rust::api::chat::ChatStreamResponse::ChatTextGeneration { text, .. }) = res {
                                    let lines: Vec<&str> = text.split('\n').collect();
                                    for (i, line) in lines.iter().enumerate() {
                                        if !line.is_empty() {
                                            type_with_cursor_effect(line, 10, &skin);
                                        }
                                        if i < lines.len() - 1 {
                                            println!();
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Cohere chat streaming failed: {}", e);
                        }
                    }
                }

                #[cfg(feature = "hf")]
                ClientType::HuggingFace(hf_client) => {
                    let model_id = autogpt::common::utils::hf_model_from_str(&hf_client.model);
                    match hf_client.client.inference().create(&prompt, model_id).await {
                        Ok(result) => {
                            use api_huggingface::components::inference_shared::InferenceResponse;
                            let text = match result {
                                InferenceResponse::Single(o) => o.generated_text,
                                InferenceResponse::Batch(mut v) => {
                                    v.pop().map(|o| o.generated_text).unwrap_or_default()
                                }
                                _ => String::new(),
                            };
                            for word in text.split_whitespace() {
                                type_with_cursor_effect(&format!("{word} "), 20, &skin);
                            }
                        }
                        Err(e) => error!("HuggingFace inference failed: {}", e),
                    }
                }

                #[allow(unreachable_patterns)]
                _ => {
                    return Err(anyhow!(
                        "No valid AI client configured. Enable `hf`, `co`, `gem`, `oai`, `cld`, or `xai` feature."
                    ));
                }
            }
        } else if args.command.is_none() && args.net {
            #[cfg(feature = "net")]
            {
                // --net flag: connect to orchgpt orchestrator.
                info!(
                    "{}",
                    "[*] \"AGI\": 🌐 Networking mode - connecting to orchestrator..."
                        .bright_green()
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
            }
            #[cfg(not(feature = "net"))]
            {
                anyhow::bail!(
                    "Networking feature 'net' is not enabled. Recompile with --features net."
                );
            }
        } else if args.command.is_none() {
            let mixture = {
                #[cfg(feature = "mop")]
                {
                    args.mixture
                }
                #[cfg(not(feature = "mop"))]
                {
                    false
                }
            };

            let settings_mgr = SettingsManager::new();
            let mut settings = settings_mgr.load().unwrap_or_default();
            if args.yolo {
                settings.yolo = true;
            }
            if args.no_internet {
                settings.internet_access = false;
            }

            let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<TuiEvent>();
            let (input_tx, input_rx) = tokio::sync::mpsc::channel::<String>(32);
            let abort_token = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

            let yolo = settings.yolo;
            let internet_access = settings.internet_access;
            let session_id = args.session.clone();
            let workspace = args.workspace.clone();
            let tx_clone = event_tx.clone();
            let abort_clone = abort_token.clone();

            tokio::spawn(async move {
                if let Err(e) = run_generic_agent_loop(GenericAgentLoopConfig {
                    yolo,
                    internet_access,
                    session_id,
                    mixture,
                    custom_workspace: workspace,
                    event_tx: Some(tx_clone),
                    input_rx: Some(input_rx),
                    abort_token: Some(abort_clone),
                })
                .await
                {
                    eprintln!("Agent error: {e}");
                }
            });

            match TuiApp::new(event_rx, &settings, abort_token, update_available) {
                Ok(app) => {
                    if let Err(e) = app.run(input_tx) {
                        eprintln!("TUI error: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to start TUI: {e}. Falling back to text mode.");
                    run_generic_agent_loop(GenericAgentLoopConfig {
                        yolo: args.yolo,
                        internet_access: !args.no_internet,
                        session_id: args.session,
                        mixture,
                        custom_workspace: args.workspace,
                        event_tx: None,
                        input_rx: None,
                        abort_token: None,
                    })
                    .await?;
                }
            }
        } else if let Some(command) = args.command {
            #[cfg(feature = "gpt")]
            let workspace = var("AUTOGPT_WORKSPACE").unwrap_or_else(|_| "workspace/".to_string());
            #[cfg(feature = "gpt")]
            let language = "python";

            #[cfg(all(feature = "gpt", feature = "git"))]
            let mut _git_agent: Option<GitGPT> = None;
            #[cfg(feature = "gpt")]
            let mut _optimizer_gpt = OptimizerGPT::default();

            #[cfg(feature = "gpt")]
            let is_non_agentic = match command {
                Commands::Test
                | Commands::Run { .. }
                | Commands::Build { .. }
                | Commands::New { .. } => true,
                #[cfg(all(feature = "cli", feature = "mcp"))]
                Commands::Mcp { .. } => true,
                _ => false,
            };

            #[cfg(feature = "gpt")]
            if !is_non_agentic {
                #[cfg(all(feature = "gpt", feature = "git"))]
                {
                    _git_agent = Some(GitGPT::new("GitGPT", "Commit all changes").await);
                }
                let behavior =
                    "Expertise lies in modularizing monolithic source code into clean components";
                let persona = "OptimizerGPT";
                _optimizer_gpt = OptimizerGPT::new(persona, behavior, language).await;
            }
            match command {
                #[cfg(feature = "gpt")]
                Commands::Man => {
                    let behavior = "Expertise at managing projects at scale";
                    let persona = "ManagerGPT";
                    #[allow(unused_assignments)]
                    let mut manager = ManagerGPT::new(persona, behavior, "", language);

                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        #[allow(unused_mut)]
                        let mut input = read_user_input()?;

                        #[cfg(feature = "mop")]
                        if args.mixture
                            && !input.is_empty()
                            && let Some((provider, response)) = run_mixture(&input).await
                        {
                            info!(
                                "{}",
                                format!("[*] MoP: selected response from {}", provider)
                                    .bright_green()
                                    .bold()
                            );
                            let skin = MadSkin::default();
                            skin.print_text(&response);
                            input = format!(
                                "High-quality context from {provider}: {response}\n\nTask: {input}"
                            );
                        }

                        manager = ManagerGPT::new(persona, behavior, &input, language);

                        if !input.is_empty() {
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );

                            let mut task = Task {
                                description: input.clone().into(),
                                ..Default::default()
                            };
                            let _ = manager.execute(&mut task, true, true, 3).await;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                                .bright_yellow()
                                                .bold());
                        }
                    }
                }
                #[cfg(feature = "gpt")]
                Commands::Arch => {
                    let behavior = "Expertise at managing projects at scale";
                    let persona = "ArchitectGPT";
                    #[cfg(all(feature = "gpt", feature = "git"))]
                    let mut git_agent = GitGPT::new("GitGPT", "Commit all changes").await;
                    let mut architect_agent = ArchitectGPT::new(persona, behavior).await;
                    let workspace = workspace.clone() + "architect";
                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        #[allow(unused_mut)]
                        let mut input = read_user_input()?;

                        #[cfg(feature = "mop")]
                        if args.mixture
                            && !input.is_empty()
                            && let Some((provider, response)) = run_mixture(&input).await
                        {
                            info!(
                                "{}",
                                format!("[*] MoP: selected response from {}", provider)
                                    .bright_green()
                                    .bold()
                            );
                            let skin = MadSkin::default();
                            skin.print_text(&response);
                            input = format!(
                                "High-quality context from {provider}: {response}\n\nTask: {input}"
                            );
                        }

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
                            let mut task = Task {
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
                                .execute(&mut task, true, false, 3)
                                .await
                                .unwrap();
                            info!(
                                "{}",
                                "[*] \"AGI\": Committing the new code to Git..."
                                    .green()
                                    .bold()
                            );
                            #[cfg(all(feature = "gpt", feature = "git"))]
                            {
                                let _ = git_agent.execute(&mut task, true, false, 1).await;
                            }
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
                #[cfg(feature = "gpt")]
                Commands::Front => {
                    let behavior = "Expertise lies in writing frontend code";
                    let persona = "FrontendGPT";
                    #[cfg(all(feature = "gpt", feature = "git"))]
                    let mut git_agent = GitGPT::new("GitGPT", "Commit all changes").await;
                    let workspace = workspace.clone() + "frontend";
                    let mut frontend_agent = FrontendGPT::new(persona, behavior, language).await;

                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        #[allow(unused_mut)]
                        let mut input = read_user_input()?;

                        #[cfg(feature = "mop")]
                        if args.mixture
                            && !input.is_empty()
                            && let Some((provider, response)) = run_mixture(&input).await
                        {
                            info!(
                                "{}",
                                format!("[*] MoP: selected response from {}", provider)
                                    .bright_green()
                                    .bold()
                            );
                            let skin = MadSkin::default();
                            skin.print_text(&response);
                            input = format!(
                                "High-quality context from {provider}: {response}\n\nTask: {input}"
                            );
                        }

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );

                            let mut task = Task {
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
                                .execute(&mut task, true, false, 3)
                                .await
                                .unwrap();
                            info!(
                                "{}",
                                "[*] \"AGI\": Committing the new code to Git..."
                                    .green()
                                    .bold()
                            );
                            #[cfg(all(feature = "gpt", feature = "git"))]
                            {
                                let _ = git_agent.execute(&mut task, true, false, 1).await;
                            }
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
                #[cfg(feature = "gpt")]
                Commands::Back => {
                    let behavior =
                        "Expertise lies in writing backend code for web servers and databases";
                    let persona = "BackendGPT";
                    #[cfg(all(feature = "gpt", feature = "git"))]
                    let mut git_agent = GitGPT::new("GitGPT", "Commit all changes").await;
                    let workspace = workspace.clone() + "backend";
                    let mut backend_gpt = BackendGPT::new(persona, behavior, language).await;

                    let mut task = Task {
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
                        #[allow(unused_mut)]
                        let mut input = read_user_input()?;

                        #[cfg(feature = "mop")]
                        if args.mixture
                            && !input.is_empty()
                            && let Some((provider, response)) = run_mixture(&input).await
                        {
                            info!(
                                "{}",
                                format!("[*] MoP: selected response from {}", provider)
                                    .bright_green()
                                    .bold()
                            );
                            let skin = MadSkin::default();
                            skin.print_text(&response);
                            input = format!(
                                "High-quality context from {provider}: {response}\n\nTask: {input}"
                            );
                        }

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
                            task.description = input.into();

                            backend_gpt
                                .execute(&mut task, true, false, 3)
                                .await
                                .unwrap();
                            info!(
                                "{}",
                                "[*] \"AGI\": Committing the new code to Git..."
                                    .green()
                                    .bold()
                            );
                            #[cfg(all(feature = "gpt", feature = "git"))]
                            {
                                let _ = git_agent.execute(&mut task, true, false, 1).await;
                            }
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
                #[cfg(feature = "gpt")]
                Commands::Design => {
                    let behavior = "Crafts stunning web design layouts";
                    let persona = "Web Designer";
                    #[cfg(all(feature = "gpt", feature = "git"))]
                    let mut git_agent = GitGPT::new("GitGPT", "Commit all changes").await;
                    let mut designer_agent = DesignerGPT::new(persona, behavior).await;
                    let mut task = Task {
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
                        #[allow(unused_mut)]
                        let mut input = read_user_input()?;

                        #[cfg(feature = "mop")]
                        if args.mixture
                            && !input.is_empty()
                            && let Some((provider, response)) = run_mixture(&input).await
                        {
                            info!(
                                "{}",
                                format!("[*] MoP: selected response from {}", provider)
                                    .bright_green()
                                    .bold()
                            );
                            let skin = MadSkin::default();
                            skin.print_text(&response);
                            input = format!(
                                "High-quality context from {provider}: {response}\n\nTask: {input}"
                            );
                        }

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
                            task.description = input.into();

                            designer_agent.execute(&mut task, true, false, 3).await?;

                            info!(
                                "{}",
                                "[*] \"AGI\": Committing the new files to Git..."
                                    .green()
                                    .bold()
                            );

                            #[cfg(all(feature = "gpt", feature = "git"))]
                            {
                                git_agent.execute(&mut task, true, false, 1).await?;
                            }
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                                    .bright_yellow()
                                                    .bold());
                        }
                    }
                }
                #[cfg(all(feature = "gpt", feature = "mail"))]
                Commands::Mail => {
                    let behavior = "Expertise at summarizing emails";
                    let persona = "Mailer";
                    let mut mailer_agent = MailerGPT::new(persona, behavior).await;
                    let mut task = Task {
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
                        #[allow(unused_mut)]
                        let mut input = read_user_input()?;

                        #[cfg(feature = "mop")]
                        if args.mixture
                            && !input.is_empty()
                            && let Some((provider, response)) = run_mixture(&input).await
                        {
                            info!(
                                "{}",
                                format!("[*] MoP: selected response from {}", provider)
                                    .bright_green()
                                    .bold()
                            );
                            let skin = MadSkin::default();
                            skin.print_text(&response);
                            input = format!(
                                "High-quality context from {provider}: {response}\n\nTask: {input}"
                            );
                        }

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
                            task.description = input.into();

                            #[cfg(all(feature = "gpt", feature = "mail"))]
                            {
                                let _ = mailer_agent.execute(&mut task, true, false, 3).await;
                            }
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                                    .bright_yellow()
                                                    .bold());
                        }
                    }
                }
                #[cfg(all(feature = "gpt", feature = "git"))]
                Commands::Git => {
                    let behavior = "Expertise at managing git repositories";
                    let persona = "GitGPT";
                    let mut git_agent = GitGPT::new(persona, behavior).await;
                    let mut task = Task {
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
                        #[allow(unused_mut)]
                        let mut input = read_user_input()?;

                        #[cfg(feature = "mop")]
                        if args.mixture
                            && !input.is_empty()
                            && let Some((provider, response)) = run_mixture(&input).await
                        {
                            info!(
                                "{}",
                                format!("[*] MoP: selected response from {}", provider)
                                    .bright_green()
                                    .bold()
                            );
                            let skin = MadSkin::default();
                            skin.print_text(&response);
                            input = format!(
                                "High-quality context from {provider}: {response}\n\nTask: {input}"
                            );
                        }

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
                            task.description = input.into();

                            #[cfg(all(feature = "gpt", feature = "git"))]
                            {
                                let _result = git_agent.execute(&mut task, true, false, 1).await;
                            }
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                                    .bright_yellow()
                                                    .bold());
                        }
                    }
                }
                #[cfg(feature = "gpt")]
                Commands::Opt => {
                    let behavior = "Expertise at optimizing codebases";
                    let persona = "OptimizerGPT";
                    let mut optimizer_agent = OptimizerGPT::new(persona, behavior, language).await;

                    let mut task = Task {
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
                            task.description = input.into();

                            let _result = optimizer_agent.execute(&mut task, true, false, 1).await;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                                    .bright_yellow()
                                                    .bold());
                        }
                    }
                }
                #[cfg(not(feature = "gpt"))]
                Commands::Man
                | Commands::Arch
                | Commands::Front
                | Commands::Back
                | Commands::Design
                | Commands::Mail
                | Commands::Git
                | Commands::Opt => {
                    anyhow::bail!(
                        "Agentic commands require the 'gpt' feature. Recompile with --features gpt."
                    );
                }
                #[cfg(all(feature = "gpt", not(feature = "mail")))]
                Commands::Mail => {
                    anyhow::bail!("Mail feature required. Recompile with --features mail.");
                }
                #[cfg(all(feature = "gpt", not(feature = "git")))]
                Commands::Git => {
                    anyhow::bail!("Git feature required. Recompile with --features git.");
                }
                Commands::New { name, feature } => new::handle_new(&name, feature)?,
                Commands::Build { out } => build::handle_build(out)?,
                Commands::Run { feature } => run::handle_run(feature.unwrap_or_default())?,
                Commands::Test => test::handle_test()?,
                #[cfg(all(feature = "cli", feature = "mcp"))]
                Commands::Mcp { subcommand } => match subcommand {
                    McpSubcommand::Add(args) => {
                        tokio::task::spawn_blocking(move || {
                            mcp_cmd::cmd_mcp_add(
                                &args.name,
                                &args.command_or_url,
                                args.args,
                                &args.transport,
                                args.env_pairs,
                                args.headers,
                                args.timeout,
                                args.trust,
                                args.description,
                                args.include_tools,
                                args.exclude_tools,
                            )
                        })
                        .await??;
                    }
                    McpSubcommand::List => {
                        tokio::task::spawn_blocking(mcp_cmd::cmd_mcp_list).await??;
                    }
                    McpSubcommand::Remove { name } => {
                        tokio::task::spawn_blocking(move || mcp_cmd::cmd_mcp_remove(&name))
                            .await??;
                    }
                    McpSubcommand::Inspect { name } => {
                        tokio::task::spawn_blocking(move || mcp_cmd::cmd_mcp_inspect(&name))
                            .await??;
                    }
                    McpSubcommand::Call { server, tool, args } => {
                        tokio::task::spawn_blocking(move || {
                            mcp_cmd::cmd_mcp_call(&server, &tool, args)
                        })
                        .await??;
                    }
                },
            };
        }
    }
    Ok(())
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
