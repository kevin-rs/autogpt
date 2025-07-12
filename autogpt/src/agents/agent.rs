//! # `AgentGPT` agent.
//!

use crate::common::utils::{
    Capability, Communication, ContextManager, Knowledge, Persona, Planner, Reflection, Status,
    Task, TaskScheduler, Tool, default_eval_fn,
};
use crate::traits::agent::Agent;
use derivative::Derivative;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
#[cfg(feature = "net")]
use {
    crate::collaboration::{AgentNet, Collaborator, RemoteAgent, delegate_task},
    crate::common::utils::AgentMessage,
    crate::traits::functions::Collaborate,
    anyhow::{Result, anyhow},
    async_trait::async_trait,
    iac_rs::prelude::*,
    std::collections::VecDeque,
    std::sync::Arc,
    std::time::Duration,
    tokio::sync::Mutex,
};

/// Represents an agent with memory, tools, and other autonomous capabilities.
#[derive(Derivative)]
#[derivative(PartialEq, Debug, Clone)]
pub struct AgentGPT {
    /// Unique identifier for the agent.
    pub id: Cow<'static, str>,

    /// The objective or mission of the agent.
    pub objective: Cow<'static, str>,

    /// The logical or physical position of the agent.
    pub position: Cow<'static, str>,

    /// The current operational status of the agent.
    pub status: Status,

    /// Hot memory containing past communications.
    pub memory: Vec<Communication>,

    /// Tools available to the agent.
    pub tools: Vec<Tool>,

    /// Structured knowledge base used for reasoning or retrieval.
    pub knowledge: Knowledge,

    /// Optional planner to manage goal sequencing.
    pub planner: Option<Planner>,

    /// Persona defines behavior style and traits.
    pub persona: Persona,

    /// Optional self-reflection module for introspection or evaluation.
    pub reflection: Option<Reflection>,

    /// Optional task scheduler for time-based goal management.
    pub scheduler: Option<TaskScheduler>,

    /// Capabilities this agent has access to (e.g. CodeGen, WebSearch).
    pub capabilities: HashSet<Capability>,

    /// Manages context for conversation and topic focus.
    pub context: ContextManager,

    /// List of tasks assigned to this agent.
    pub tasks: Vec<Task>,

    /// Cryptographic signer for agent authentication and message integrity.
    #[cfg(feature = "net")]
    pub signer: Signer,

    /// Map of verifier instances used to verify signatures from peers.
    #[cfg(feature = "net")]
    pub verifiers: HashMap<String, Verifier>,

    /// Network address this agent binds to for communication (e.g., "0.0.0.0:8080").
    #[cfg(feature = "net")]
    pub addr: String,

    /// Connected client sessions to peer agents.
    #[cfg(feature = "net")]
    #[derivative(PartialEq = "ignore")]
    pub clients: HashMap<String, Arc<Mutex<Client>>>,

    /// Optional server instance handling incoming peer connections.
    #[cfg(feature = "net")]
    #[derivative(PartialEq = "ignore")]
    pub server: Option<Arc<Mutex<Server>>>,

    /// Interval for sending heartbeat signals to peers for liveness detection.
    #[cfg(feature = "net")]
    pub heartbeat_interval: Duration,

    /// Map of peer agent identifiers to their network addresses.
    #[cfg(feature = "net")]
    pub peer_addresses: HashMap<String, String>,

    /// Other agents this agent collaborates with, running in the same memory
    /// space/thread or within the same runtime.
    #[cfg(feature = "net")]
    #[derivative(PartialEq = "ignore")]
    pub local_collaborators: HashMap<String, Collaborator>,

    /// Other agents this agent collaborates with via the network, using
    /// inter/intra-agent communication (IAC) protocols.
    #[cfg(feature = "net")]
    #[derivative(PartialEq = "ignore")]
    pub remote_collaborators: HashMap<String, Collaborator>,

    /// Maps capabilities to a round-robin queue of peer agent IDs
    /// for distributing tasks across collaborators.
    #[cfg(feature = "net")]
    pub cap_index: HashMap<Capability, VecDeque<String>>,

    /// Round-robin index used to evenly distribute workload among peers.
    #[cfg(feature = "net")]
    pub rr_idx: usize,
}

impl Default for AgentGPT {
    fn default() -> Self {
        Self {
            id: Cow::Owned(Uuid::new_v4().to_string()),
            objective: Cow::Borrowed(""),
            position: Cow::Borrowed(""),
            status: Status::default(),
            memory: vec![],
            tools: vec![],
            knowledge: Knowledge::default(),
            planner: None,
            persona: Persona {
                name: Cow::Borrowed("Default"),
                traits: vec![],
                behavior_script: None,
            },
            reflection: None,
            scheduler: None,
            capabilities: HashSet::new(),
            context: ContextManager {
                recent_messages: vec![],
                focus_topics: vec![],
            },
            tasks: vec![],
            #[cfg(feature = "net")]
            signer: Signer::new(KeyPair::generate()),
            #[cfg(feature = "net")]
            verifiers: HashMap::new(),
            #[cfg(feature = "net")]
            addr: "0.0.0.0:0".to_string(),
            #[cfg(feature = "net")]
            clients: HashMap::new(),
            #[cfg(feature = "net")]
            server: None,
            #[cfg(feature = "net")]
            heartbeat_interval: Duration::from_secs(30),
            #[cfg(feature = "net")]
            peer_addresses: HashMap::new(),
            #[cfg(feature = "net")]
            local_collaborators: HashMap::new(),
            #[cfg(feature = "net")]
            remote_collaborators: HashMap::new(),
            #[cfg(feature = "net")]
            cap_index: HashMap::new(),
            #[cfg(feature = "net")]
            rr_idx: 0,
        }
    }
}

impl AgentGPT {
    /// Adds a communication to the memory of the agent.
    ///
    /// # Arguments
    ///
    /// * `communication` - The communication to be added to the memory.
    pub fn add_communication(&mut self, communication: Communication) {
        self.memory.push(communication);
    }

    /// Creates a new instance of `AgentGPT` with owned strings.
    ///
    /// # Arguments
    ///
    /// * `objective` - The objective of the agent.
    /// * `position` - The position of the agent.
    ///
    /// # Returns
    ///
    /// A new fully initialized instance of `AgentGPT`.
    pub fn new_owned(objective: String, position: String) -> Self {
        Self {
            id: Cow::Owned(Uuid::new_v4().to_string()),
            objective: Cow::Owned(objective),
            position: Cow::Owned(position.clone()),
            status: Status::Idle,

            memory: vec![],

            tools: vec![],

            knowledge: Knowledge {
                facts: HashMap::default(),
            },

            planner: Some(Planner {
                current_plan: vec![],
            }),

            persona: Persona {
                name: position.into(),
                traits: vec![],
                behavior_script: None,
            },

            reflection: Some(Reflection {
                recent_logs: vec![],
                evaluation_fn: default_eval_fn,
            }),

            scheduler: Some(TaskScheduler {
                scheduled_tasks: vec![],
            }),

            capabilities: HashSet::default(),

            context: ContextManager {
                recent_messages: vec![],
                focus_topics: vec![],
            },

            tasks: vec![],
            #[cfg(feature = "net")]
            signer: Signer::new(KeyPair::generate()),
            #[cfg(feature = "net")]
            verifiers: HashMap::new(),
            #[cfg(feature = "net")]
            addr: "0.0.0.0:0".to_string(),
            #[cfg(feature = "net")]
            clients: HashMap::new(),
            #[cfg(feature = "net")]
            server: None,
            #[cfg(feature = "net")]
            heartbeat_interval: Duration::from_secs(30),
            #[cfg(feature = "net")]
            peer_addresses: HashMap::new(),
            #[cfg(feature = "net")]
            local_collaborators: HashMap::new(),
            #[cfg(feature = "net")]
            remote_collaborators: HashMap::new(),
            #[cfg(feature = "net")]
            cap_index: HashMap::new(),
            #[cfg(feature = "net")]
            rr_idx: 0,
        }
    }

    /// Creates a new instance of `AgentGPT` with borrowed string slices.
    ///
    /// # Arguments
    ///
    /// * `objective` - The objective of the agent.
    /// * `position` - The position of the agent.
    ///
    /// # Returns
    ///
    /// A new fully initialized instance of `AgentGPT`.
    pub fn new_borrowed(objective: &'static str, position: &'static str) -> Self {
        Self {
            id: Cow::Owned(Uuid::new_v4().to_string()),
            objective: Cow::Borrowed(objective),
            position: Cow::Borrowed(position),
            status: Status::Idle,

            memory: vec![],

            tools: vec![],

            knowledge: Knowledge {
                facts: HashMap::default(),
            },

            planner: Some(Planner {
                current_plan: vec![],
            }),

            persona: Persona {
                name: position.into(),
                traits: vec![],
                behavior_script: None,
            },

            reflection: Some(Reflection {
                recent_logs: vec![],
                evaluation_fn: default_eval_fn,
            }),

            scheduler: Some(TaskScheduler {
                scheduled_tasks: vec![],
            }),

            capabilities: HashSet::default(),

            context: ContextManager {
                recent_messages: vec![],
                focus_topics: vec![],
            },

            tasks: vec![],
            #[cfg(feature = "net")]
            signer: Signer::new(KeyPair::generate()),
            #[cfg(feature = "net")]
            verifiers: HashMap::new(),
            #[cfg(feature = "net")]
            addr: "0.0.0.0:0".to_string(),
            #[cfg(feature = "net")]
            clients: HashMap::new(),
            #[cfg(feature = "net")]
            server: None,
            #[cfg(feature = "net")]
            heartbeat_interval: Duration::from_secs(30),
            #[cfg(feature = "net")]
            peer_addresses: HashMap::new(),
            #[cfg(feature = "net")]
            local_collaborators: HashMap::new(),
            #[cfg(feature = "net")]
            remote_collaborators: HashMap::new(),
            #[cfg(feature = "net")]
            cap_index: HashMap::new(),
            #[cfg(feature = "net")]
            rr_idx: 0,
        }
    }

    #[cfg(feature = "net")]
    pub async fn register_local(&mut self, collab: Collaborator, caps: Vec<Capability>) {
        let id = collab.id().await;
        self.local_collaborators.insert(id.clone(), collab);
        for cap in caps {
            self.cap_index.entry(cap).or_default().push_back(id.clone());
        }
    }

    #[cfg(feature = "net")]
    pub fn register_remote(&mut self, id: Cow<'static, str>, caps: Vec<Capability>) {
        let remote = Collaborator::Remote(RemoteAgent {
            id: id.clone(),
            signer: self.signer.clone(),
            clients: self.clients.clone(),
        });

        self.remote_collaborators
            .insert(id.to_string(), remote.clone());

        for cap in caps {
            self.cap_index
                .entry(cap)
                .or_default()
                .push_back(id.to_string());
        }
    }

    #[cfg(feature = "net")]
    pub async fn assign_task_lb(&mut self, cap: &Capability, task: Task) -> Result<()> {
        let queue = self
            .cap_index
            .get_mut(cap)
            .ok_or_else(|| anyhow!("No agent has capability: {:?}", cap))?;

        let id = queue[self.rr_idx % queue.len()].clone();
        self.rr_idx += 1;

        let collab = self
            .local_collaborators
            .get(&id)
            .or(self.remote_collaborators.get(&id))
            .ok_or_else(|| anyhow!("Collaborator with id {} not found", id))?;

        delegate_task(collab.clone(), task).await
    }
    #[cfg(feature = "net")]
    pub fn as_agent_net(&self) -> AgentNet {
        AgentNet {
            id: self.id.clone(),
            signer: self.signer.clone(),
            verifiers: self.verifiers.clone(),
            addr: self.addr.clone(),
            clients: self.clients.clone(),
            server: self.server.clone(),
            heartbeat_interval: self.heartbeat_interval,
            peer_addresses: self.peer_addresses.clone(),
        }
    }
}

impl Agent for AgentGPT {
    /// Creates a new `AgentGPT` instance with the given objective and position.
    fn new(objective: Cow<'static, str>, position: Cow<'static, str>) -> Self {
        Self {
            id: Cow::Owned(Uuid::new_v4().to_string()),

            objective,
            position: position.clone(),
            status: Status::Idle,

            memory: vec![],

            tools: vec![],

            knowledge: Knowledge {
                facts: HashMap::default(),
            },

            planner: Some(Planner {
                current_plan: vec![],
            }),

            persona: Persona {
                name: position,
                traits: vec![],
                behavior_script: None,
            },

            reflection: Some(Reflection {
                recent_logs: vec![],
                evaluation_fn: default_eval_fn,
            }),

            scheduler: Some(TaskScheduler {
                scheduled_tasks: vec![],
            }),

            capabilities: HashSet::default(),

            context: ContextManager {
                recent_messages: vec![],
                focus_topics: vec![],
            },

            tasks: vec![],
            #[cfg(feature = "net")]
            signer: Signer::new(KeyPair::generate()),
            #[cfg(feature = "net")]
            verifiers: HashMap::new(),
            #[cfg(feature = "net")]
            addr: "0.0.0.0:0".to_string(),
            #[cfg(feature = "net")]
            clients: HashMap::new(),
            #[cfg(feature = "net")]
            server: None,
            #[cfg(feature = "net")]
            heartbeat_interval: Duration::from_secs(30),
            #[cfg(feature = "net")]
            peer_addresses: HashMap::new(),
            #[cfg(feature = "net")]
            local_collaborators: HashMap::new(),
            #[cfg(feature = "net")]
            remote_collaborators: HashMap::new(),
            #[cfg(feature = "net")]
            cap_index: HashMap::new(),
            #[cfg(feature = "net")]
            rr_idx: 0,
        }
    }

    /// Updates the agent's operational status.
    fn update(&mut self, status: Status) {
        self.status = status;
    }

    /// Returns the agent's objective.
    fn objective(&self) -> &Cow<'static, str> {
        &self.objective
    }

    /// Returns the agent's current position.
    fn position(&self) -> &Cow<'static, str> {
        &self.position
    }

    /// Returns the agent's current status.
    fn status(&self) -> &Status {
        &self.status
    }

    /// Returns the agent's memory log of communications.
    fn memory(&self) -> &Vec<Communication> {
        &self.memory
    }

    /// Returns the agent's available tools.
    fn tools(&self) -> &Vec<Tool> {
        &self.tools
    }

    /// Returns the agent's structured knowledge base.
    fn knowledge(&self) -> &Knowledge {
        &self.knowledge
    }

    /// Returns an optional reference to the agent's planner.
    fn planner(&self) -> Option<&Planner> {
        self.planner.as_ref()
    }

    /// Returns the agent's persona configuration.
    fn persona(&self) -> &Persona {
        &self.persona
    }

    /// Returns a list of agents this agent collaborates with.
    #[cfg(feature = "net")]
    fn collaborators(&self) -> Vec<Collaborator> {
        let mut all = Vec::new();
        all.extend(self.local_collaborators.values().cloned());
        all.extend(self.remote_collaborators.values().cloned());
        all
    }

    /// Returns an optional reference to the self-reflection module.
    fn reflection(&self) -> Option<&Reflection> {
        self.reflection.as_ref()
    }

    /// Returns an optional reference to the agent's task scheduler.
    fn scheduler(&self) -> Option<&TaskScheduler> {
        self.scheduler.as_ref()
    }

    /// Returns the agent's registered capabilities.
    fn capabilities(&self) -> &HashSet<Capability> {
        &self.capabilities
    }

    /// Returns the context manager tracking recent communication and focus.
    fn context(&self) -> &ContextManager {
        &self.context
    }

    /// Returns the list of current tasks or tasks.
    fn tasks(&self) -> &Vec<Task> {
        &self.tasks
    }

    fn memory_mut(&mut self) -> &mut Vec<Communication> {
        &mut self.memory
    }

    fn planner_mut(&mut self) -> Option<&mut Planner> {
        self.planner.as_mut()
    }

    fn context_mut(&mut self) -> &mut ContextManager {
        &mut self.context
    }
}

#[cfg(feature = "net")]
impl AgentGPT {
    pub async fn broadcast_capabilities(&self) -> Result<()> {
        let msg = AgentMessage::CapabilityAdvert {
            sender_id: self.id.to_string(),
            capabilities: self.capabilities.iter().cloned().collect(),
        };

        let payload = serde_json::to_vec(&msg)?;

        for (peer_id, client) in &self.clients {
            let mut message = Message {
                from: self.id.clone().into(),
                to: peer_id.clone(),
                msg_type: MessageType::Broadcast,
                extra_data: payload.clone(),
                ..Default::default()
            };

            message.sign(&self.signer)?;
            client.lock().await.send(message).await?;
        }

        Ok(())
    }
}

#[async_trait]
#[cfg(feature = "net")]
impl Collaborate for AgentGPT {
    async fn handle_task(&mut self, task: Task) -> Result<()> {
        // TODO: implement this func
        let mut this = self.clone();
        this.tasks.push(task);
        Ok(())
    }

    async fn receive_message(&mut self, msg: AgentMessage) -> Result<()> {
        match msg {
            AgentMessage::Task(task) => self.handle_task(task).await,

            AgentMessage::CapabilityAdvert {
                sender_id,
                capabilities,
            } => {
                self.register_remote(sender_id.into(), capabilities);
                Ok(())
            }

            _ => Ok(()),
        }
    }

    fn get_id(&self) -> &str {
        &self.id
    }
}

#[async_trait]
#[cfg(feature = "net")]
impl Network for AgentGPT {
    async fn heartbeat(&self) {
        let clients = self.clients.clone();
        let peer_addresses = self.peer_addresses.clone();
        let signer = self.signer.clone();
        let id = self.id.to_string();
        let interval = self.heartbeat_interval;

        tokio::spawn(async move {
            loop {
                for (peer_id, client) in &clients {
                    let msg = Message::ping(&id, peer_id, 0);
                    let result = {
                        let client = client.lock().await;
                        client.send(msg).await
                    };

                    if let Err(e) = result {
                        debug!("Heartbeat failed to {peer_id}: {e}");

                        if let Some(addr) = peer_addresses.get(peer_id) {
                            debug!("Attempting to reconnect to {peer_id} at {addr}...");

                            match Client::connect(addr, signer.clone()).await {
                                Ok(new_client) => {
                                    debug!("Reconnected to {peer_id}");
                                    let mut locked = client.lock().await;
                                    *locked = new_client;
                                }
                                Err(err) => {
                                    debug!("Failed to reconnect to {peer_id}: {err}");
                                }
                            }
                        } else {
                            debug!("No known address for {peer_id}, cannot reconnect.");
                        }
                    }
                }

                tokio::time::sleep(interval).await;
            }
        });
    }

    async fn broadcast(&self, payload: &str) -> anyhow::Result<()> {
        let tasks = self.clients.iter().map(|(peer_id, client)| {
            let mut msg = Message::broadcast(&self.id, payload, 0);
            msg.to = peer_id.clone();
            let client = client.clone();
            async move {
                let send_result = {
                    let client_guard = client.lock().await;
                    client_guard.clone()
                }
                .send(msg)
                .await;

                if let Err(e) = send_result {
                    debug!("Broadcast to {peer_id} failed: {e}");
                } else {
                    debug!("Broadcast to {peer_id} succeeded");
                }
            }
        });

        futures::future::join_all(tasks).await;
        Ok(())
    }
}
