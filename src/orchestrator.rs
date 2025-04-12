use std::collections::HashMap;
use std::sync::Arc;

use tokio::{
    io::AsyncReadExt,
    net::TcpListener,
    sync::{mpsc, Mutex},
};
use tracing::{debug, error, info, warn};

use crate::agents::architect::ArchitectGPT;
use crate::agents::backend::BackendGPT;
#[cfg(feature = "img")]
use crate::agents::designer::DesignerGPT;
use crate::agents::frontend::FrontendGPT;
use crate::agents::git::GitGPT;
use crate::agents::types::AgentType;
use crate::{
    common::tls::load_tls_config,
    message::{parse_message, Message},
};
use std::env;

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
        info!("[Orchestrator] Listening on {}", bind_address);

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
                        let n = match tls_stream.read(&mut buf).await {
                            Ok(n) => n,
                            Err(e) => {
                                error!("TLS read error: {}", e);
                                return;
                            }
                        };
                        if let Ok(msg) = parse_message(&buf[..n]) {
                            let mut agents = agents.lock().await;
                            if let Some(agent) = agents.get_mut(&msg.to) {
                                debug!("Agent {:?} found", agent.position());
                                // TODO: Fix `gems` client and make it threads safe cz of dyn std::error::Error
                                // let mut tasks = Tasks::from_payload(&msg.payload_json);
                                // let _ = agent.execute(&mut tasks, true, false, 3).await;
                            } else {
                                warn!("Agent {:?} not found", agents);
                            }
                        }
                    });
                },
                Some(msg) = self.receiver.recv() => {
                    let mut agents = self.agents.lock().await;
                    match msg.msg_type.as_str() {
                        "create" => {
                            let agent_type = match msg.to.as_str() {
                                "ArchitectGPT" => Some(AgentType::Architect(ArchitectGPT::new("Architect agent", "ArchitectGPT"))),
                                "BackendGPT" => Some(AgentType::Backend(BackendGPT::new("Backend agent", "BackendGPT", "rust"))),
                                "FrontendGPT" => Some(AgentType::Frontend(FrontendGPT::new("Frontend agent", "FrontendGPT", "rust"))),
                                #[cfg(feature = "img")]
                                "DesignerGPT" => Some(AgentType::Designer(DesignerGPT::new("Designer agent", "DesignerGPT"))),
                                #[cfg(feature = "git")]
                                "GitGPT" => Some(AgentType::Git(GitGPT::new("Git agent", "GitGPT"))),
                                _ => None,
                            };

                            if let Some(agent) = agent_type {
                                agents.insert(msg.to.clone(), agent);
                                info!("Agent {} created", msg.to);
                            } else {
                                warn!("Unknown agent type: {}", msg.to);
                            }
                        },
                        "terminate" => {
                            agents.remove(&msg.to);
                            info!("Agent {} terminated", msg.to);
                        },
                        "run" => {
                            if let Some(_agent) = agents.get_mut(&msg.to) {
                                // TODO: Fix `gems` client and make it threads safe cz of dyn std::error::Error
                                // let mut tasks = Tasks::from_payload(&msg.payload_json);
                                // let _ = agent.execute(&mut tasks, true, false, 3).await;
                                info!("Executed tasks for agent {}", msg.to);
                            } else {
                                warn!("Agent {} not found", msg.to);
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}
