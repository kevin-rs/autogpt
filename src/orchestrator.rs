use std::collections::HashMap;
use std::sync::Arc;

use tokio::{
    io::AsyncReadExt,
    net::TcpListener,
    sync::{mpsc, Mutex},
};
use tracing::{error, info, warn};

use crate::agents::architect::ArchitectGPT;
use crate::agents::backend::BackendGPT;
#[cfg(feature = "img")]
use crate::agents::designer::DesignerGPT;
use crate::agents::frontend::FrontendGPT;
use crate::agents::git::GitGPT;
use crate::agents::types::AgentType;
use crate::common::utils::Tasks;
use crate::message::parse_kv;
use crate::{
    common::tls::load_tls_config,
    message::{parse_message, Message},
};
use std::env;
use tokio::io::AsyncWriteExt;
use tokio::time::{timeout, Duration};

/// Struct representing the orchestrator responsible for managing and coordinating multiple agents.
///
/// The `Orchestrator` listens for incoming connections over TLS and processes messages received via
/// a channel. It can create, terminate, and run tasks for different types of agents based on the
/// messages it receives. The orchestrator interacts with agents via an asynchronous model using
/// the Tokio runtime.
pub struct Orchestrator {
    /// A shared, thread-safe map of agent names to their respective agent types.
    pub agents: Arc<Mutex<HashMap<String, AgentType>>>,

    /// A receiver channel that receives messages instructing the orchestrator on agent actions.
    receiver: mpsc::Receiver<Message>,
}

impl Orchestrator {
    /// Creates a new `Orchestrator` instance with a given message receiver.
    ///
    /// # Arguments
    ///
    /// * `receiver` - A receiver channel (`mpsc::Receiver<Message>`) that the orchestrator will use
    ///   to receive messages about agent actions.
    ///
    /// # Returns
    ///
    /// `Ok(Orchestrator)` - A new `Orchestrator` instance wrapped in a `Result`. If the receiver is successfully
    /// created, it returns an `Orchestrator` instance. If an error occurs during initialization,
    /// it returns an `Err` with details about the error.
    pub async fn new(receiver: mpsc::Receiver<Message>) -> anyhow::Result<Self> {
        Ok(Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            receiver,
        })
    }

    /// Runs the orchestrator, listening for incoming TLS connections and processing messages from the receiver.
    ///
    /// This method performs the main loop of the orchestrator. It listens for connections over TLS, and for each
    /// connection, it processes the incoming data, which is expected to be a message intended for one of the agents.
    /// It also processes messages received via the `receiver` channel to create, terminate, or run tasks on agents.
    ///
    /// # Returns
    ///
    /// `Ok(())` - Returns `Ok` if the orchestrator starts and runs successfully.
    ///
    /// `Err(anyhow::Error)` - Returns an error if something goes wrong while running the orchestrator.
    pub async fn run(mut self) -> anyhow::Result<()> {
        let tls_config = load_tls_config()?;
        let bind_address =
            env::var("ORCHESTRATOR_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8443".to_string());
        let listener = TcpListener::bind(bind_address.to_string()).await?;
        info!("[*] \"Orchestrator\": Listening on {}", bind_address);

        loop {
            tokio::select! {
                Ok((stream, _)) = listener.accept() => {
                    let agents = self.agents.clone();
                    let tls = tls_config.clone();
                    tokio::spawn(async move {
                        let mut tls_stream = match tls.accept(stream).await {
                            Ok(s) => s,
                            Err(e) => {
                                error!("TLS accept error: {}", e);
                                return;
                            }
                        };

                        let mut buf = [0u8; 4096];

                        loop {
                            let read_result = timeout(Duration::from_secs(60), tls_stream.read(&mut buf)).await;

                            let n = match read_result {
                                Ok(Ok(0)) => {
                                    info!("[*] \"Orchestrator\": Client disconnected gracefully.");
                                    break;
                                },
                                Ok(Ok(n)) => n,
                                Ok(Err(e)) => {
                                    let _ = tls_stream.write_all(b"[*] \"Orchestrator\": TLS read error\n").await;
                                    error!("[*] \"Orchestrator\": TLS read error: {}", e);
                                    break;
                                },
                                Err(_) => {
                                    let _ = tls_stream.write_all(b"[*] \"Orchestrator\": Read timed out\n").await;
                                    warn!("[*] \"Orchestrator\": Read timeout - closing connection.");
                                    break;
                                },
                            };

                            if let Ok(msg) = parse_message(&buf[..n]) {
                                let mut agents = agents.lock().await;

                                match msg.msg_type.as_str() {
                                    "create" => {
                                        let payload = parse_kv(&msg.payload_json);
                                        let lang_str = if payload.1.trim().is_empty() {
                                            "python".to_string()
                                        } else {
                                            payload.1
                                        };
                                        let lang: &'static str = Box::leak(lang_str.into_boxed_str());

                                        let agent_type = match msg.to.as_str() {
                                            "arch" => Some(AgentType::Architect(ArchitectGPT::new("Architect agent", "ArchitectGPT"))),
                                            "back" => Some(AgentType::Backend(BackendGPT::new("Backend agent", "BackendGPT", lang))),
                                            "front" => Some(AgentType::Frontend(FrontendGPT::new("Frontend agent", "FrontendGPT", lang))),
                                            #[cfg(feature = "img")]
                                            "design" => Some(AgentType::Designer(DesignerGPT::new("Designer agent", "DesignerGPT"))),
                                            #[cfg(feature = "git")]
                                            "git" => Some(AgentType::Git(GitGPT::new("Git agent", "GitGPT"))),
                                            _ => None,
                                        };

                                        let reply = if let Some(agent) = agent_type {
                                            agents.insert(msg.to.clone(), agent);
                                            format!("[*] \"Orchestrator\": ✅ Agent '{}' created\n", msg.to)
                                        } else {
                                            format!("[*] \"Orchestrator\": Unknown agent type '{}'\n", msg.to)
                                        };
                                        let _ = tls_stream.write_all(reply.as_bytes()).await;
                                        info!("{}", reply.trim_end());
                                    },
                                    "terminate" => {
                                        agents.remove(&msg.to);
                                        let reply = format!("[*] \"Orchestrator\": 🧹 Agent '{}' terminated\n", msg.to);
                                        let _ = tls_stream.write_all(reply.as_bytes()).await;
                                        info!("{}", reply.trim_end());
                                    },
                                    "run" => {
                                        if let Some(agent) = agents.get_mut(&msg.to) {
                                            let mut tasks = Tasks::from_payload(&msg.payload_json);
                                            let _ = agent.execute(&mut tasks, true, false, 3).await;
                                            let reply = format!("[*] \"Orchestrator\": ✅ Executed tasks for agent '{}'\n", msg.to);
                                            let _ = tls_stream.write_all(reply.as_bytes()).await;
                                            info!("{}", reply.trim_end());
                                        } else {
                                            let reply = format!("[*] \"Orchestrator\": Agent '{}' not found\n", msg.to);
                                            let _ = tls_stream.write_all(reply.as_bytes()).await;
                                            warn!("{}", reply.trim_end());
                                        }
                                    },
                                    _ => {
                                        let reply = "[*] \"Orchestrator\": ⚠️ Unknown message type\n".to_string();
                                        let _ = tls_stream.write_all(reply.as_bytes()).await;
                                        warn!("{}", reply.trim_end());
                                    }
                                }
                            } else {
                                let _ = tls_stream.write_all(b"[*] \"Orchestrator\": Failed to parse message\n").await;
                                warn!("[*] \"Orchestrator\": Failed to parse message");
                            }
                        }
                    });
                },
                Some(msg) = self.receiver.recv() => {
                    let mut agents = self.agents.lock().await;
                    match msg.msg_type.as_str() {
                        "create" => {
                            let payload = parse_kv(&msg.payload_json);
                            let lang: &'static str = Box::leak(payload.1.into_boxed_str());
                            let agent_type = match msg.to.as_str() {
                                "arch" => Some(AgentType::Architect(ArchitectGPT::new("Architect agent", "ArchitectGPT"))),
                                "back" => Some(AgentType::Backend(BackendGPT::new("Backend agent", "BackendGPT", lang))),
                                "front" => Some(AgentType::Frontend(FrontendGPT::new("Frontend agent", "FrontendGPT", lang))),
                                #[cfg(feature = "img")]
                                "design" => Some(AgentType::Designer(DesignerGPT::new("Designer agent", "DesignerGPT"))),
                                #[cfg(feature = "git")]
                                "git" => Some(AgentType::Git(GitGPT::new("Git agent", "GitGPT"))),
                                _ => None,
                            };

                            if let Some(agent) = agent_type {
                                agents.insert(msg.to.clone(), agent);
                                info!("[*] \"Orchestrator\": Agent {} created", msg.to);
                            } else {
                                warn!("[*] \"Orchestrator\": Unknown agent type: {}", msg.to);
                            }
                        },
                        "terminate" => {
                            agents.remove(&msg.to);
                            info!("[*] \"Orchestrator\": Agent {} terminated", msg.to);
                        },
                        "run" => {
                            if let Some(agent) = agents.get_mut(&msg.to) {
                                let mut tasks = Tasks::from_payload(&msg.payload_json);
                                let _ = agent.execute(&mut tasks, true, false, 3).await;
                                info!("[*] \"Orchestrator\": Executed tasks for agent {}", msg.to);
                            } else {
                                warn!("[*] \"Orchestrator\": Agent {} not found", msg.to);
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}
