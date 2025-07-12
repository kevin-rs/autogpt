use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::agents::architect::ArchitectGPT;
use crate::agents::backend::BackendGPT;
#[cfg(feature = "img")]
use crate::agents::designer::DesignerGPT;
use crate::agents::frontend::FrontendGPT;
use crate::agents::git::GitGPT;
use crate::agents::types::AgentType;
use crate::common::utils::Task;
use crate::message::parse_kv;
use iac_rs::prelude::*;
use std::env;

/// Struct representing the orchestrator responsible for managing and coordinating multiple agents.
///
/// The `Orchestrator` listens for incoming connections over TLS and processes messages received via
/// a channel. It can create, terminate, and run tasks for different types of agents based on the
/// messages it receives. The orchestrator interacts with agents via an asynchronous model using
/// the Tokio runtime.
pub struct Orchestrator {
    /// A unique identifier for the orchestrator instance.
    pub id: String,

    /// The digital signer used to sign outgoing messages.
    pub signer: Signer,

    /// The verifier used to validate incoming messages.
    pub verifier: Verifier,

    /// A shared, thread-safe map of agent names to their respective agent types.
    pub agents: Arc<Mutex<HashMap<String, AgentType>>>,
}

impl Orchestrator {
    /// Creates a new `Orchestrator` instance.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique string identifier for the orchestrator (used in message routing).
    /// * `signer` - A cryptographic signer for signing outgoing messages.
    /// * `verifier` - A cryptographic verifier for verifying incoming messages.
    ///
    /// # Returns
    ///
    /// Returns a fully initialized `Orchestrator` with an empty agent registry and
    /// no server bound yet.
    pub async fn new(id: String, signer: Signer, verifier: Verifier) -> anyhow::Result<Self> {
        Ok(Self {
            id,
            signer,
            verifier,
            agents: Arc::new(Mutex::new(HashMap::new())),
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
    pub async fn run(&mut self) -> Result<()> {
        let addr = env::var("ORCHESTRATOR_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8443".to_string());

        let agents = Arc::clone(&self.agents);
        let verifier = self.verifier.clone();
        let id = self.id.clone();
        let signer = self.signer.clone();

        let mut server = Server::bind(&addr).await?;
        info!("[*] \"Orchestrator\": Listening on {}", addr);

        let server_handle = server.clone();

        server.set_handler(move |(msg, conn)| {
            let agents = Arc::clone(&agents);
            let signer = signer.clone();
            let value = id.clone();
            let server_handle = server_handle.clone();

            async move {
                let mut agents = agents.lock().await;

                let reply = match msg.msg_type {
                    MessageType::Create => {
                        let (_input, lang) = parse_kv(&msg.payload_json);
                        let lang_str = if lang.trim().is_empty() {
                            info!("[*] \"Orchestrator\": Language not specified, defaulting to 'python'");
                            "python".to_string()
                        } else {
                            lang
                        };
                        let language = Box::leak(lang_str.into_boxed_str());

                        let new_agent = match msg.to.as_str() {
                            "arch" => {
                                info!("[*] \"Orchestrator\": Creating Architect agent '{}'", msg.to);
                                Some(AgentType::Architect(ArchitectGPT::new("Architect agent", "ArchitectGPT").await))
                            }
                            "back" => {
                                info!("[*] \"Orchestrator\": Creating Backend agent '{}', language: {}", msg.to, language);
                                Some(AgentType::Backend(BackendGPT::new("Backend agent", "BackendGPT", language).await))
                            }
                            "front" => {
                                info!("[*] \"Orchestrator\": Creating Frontend agent '{}', language: {}", msg.to, language);
                                Some(AgentType::Frontend(FrontendGPT::new("Frontend agent", "FrontendGPT", language).await))
                            }
                            #[cfg(feature = "img")]
                            "design" => {
                                info!("[*] \"Orchestrator\": Creating Designer agent '{}'", msg.to);
                                Some(AgentType::Designer(DesignerGPT::new("Designer agent", "DesignerGPT").await))
                            }
                            #[cfg(feature = "git")]
                            "git" => {
                                info!("[*] \"Orchestrator\": Creating Git agent '{}'", msg.to);
                                Some(AgentType::Git(GitGPT::new("Git agent", "GitGPT").await))
                            }
                            _ => {
                                warn!("[*] \"Orchestrator\": Unknown agent type requested '{}'", msg.to);
                                None
                            }
                        };

                        if let Some(agent) = new_agent {
                            agents.insert(msg.to.clone(), agent);
                            format!("✅ Agent '{}' created", msg.to)
                        } else {
                            format!("❌ Unknown agent type '{}'", msg.to)
                        }
                    }

                    MessageType::Terminate => {
                        if agents.remove(&msg.to).is_some() {
                            info!("[*] \"Orchestrator\": Agent '{}' terminated", msg.to);
                            format!("🧹 Agent '{}' terminated", msg.to)
                        } else {
                            warn!("[*] \"Orchestrator\": Attempted to terminate unknown agent '{}'", msg.to);
                            format!("❌ Agent '{}' not found for termination", msg.to)
                        }
                    }

                    MessageType::Run => {
                        if let Some(agent) = agents.get_mut(&msg.to) {
                            info!("[*] \"Orchestrator\": Executing tasks for agent '{}'", msg.to);
                            let mut tasks = Task::from_payload(&msg.payload_json);
                            if let Err(e) = agent.execute(&mut tasks, true, false, 3).await {
                                error!("[*] \"Orchestrator\": Error executing tasks for agent '{}': {:?}", msg.to, e);
                                format!("❌ Failed to execute tasks for agent '{}'", msg.to)
                            } else {
                                format!("✅ Executed tasks for agent '{}'", msg.to)
                            }
                        } else {
                            warn!("[*] \"Orchestrator\": Agent '{}' not found for running tasks", msg.to);
                            format!("❌ Agent '{}' not found", msg.to)
                        }
                    }

                    _ => {
                        warn!("[*] \"Orchestrator\": Unsupported message type: {:?}", msg.msg_type);
                        format!("❌ Unsupported message type: {:?}", msg.msg_type)
                    }
                };

                let response = Message::new(&value, &conn, MessageType::Reply, &reply);

                if let Err(e) = server_handle.send(&conn, response, &signer).await {
                    error!("Failed to send reply: {:?}", e);
                } else {
                    info!("[*] \"Orchestrator\": Reply sent to '{}'", conn);
                }

                Ok(())
            }
        });

        if let Err(e) = server.run(verifier).await {
            error!("[*] \"Orchestrator\": Server run error: {:?}", e);
            return Err(e);
        }

        Ok(())
    }
}
