use crate::common::utils::{AgentMessage, Capability, Task};
use crate::traits::functions::Collaborate;
use anyhow::{Result, anyhow};
use iac_rs::prelude::*;
use serde_json;
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Clone)]
pub enum Collaborator {
    Local(Arc<Mutex<dyn Collaborate>>),
    Remote(RemoteAgent),
}

impl PartialEq for Collaborator {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl fmt::Debug for Collaborator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Collaborator::Local(_) => f.debug_tuple("Local").field(&"<dyn Collaborate>").finish(),
            Collaborator::Remote(agent) => f.debug_tuple("Remote").field(agent).finish(),
        }
    }
}

impl Collaborator {
    pub async fn id(&self) -> String {
        match self {
            Collaborator::Local(loc) => {
                let guard = loc.lock().await;
                guard.get_id().to_string()
            }
            Collaborator::Remote(agent) => agent.id.to_string(),
        }
    }
}

pub async fn delegate_task(collab: &Collaborator, task: Task) -> Result<()> {
    match collab {
        Collaborator::Local(agent) => agent.lock().await.handle_task(task).await,
        Collaborator::Remote(agent) => agent.handle_task(task).await,
    }
}

#[derive(AutoNet, Clone)]
pub struct AgentNet {
    pub id: Cow<'static, str>,
    pub signer: Signer,
    pub verifiers: HashMap<String, Verifier>,
    pub addr: String,
    pub clients: HashMap<String, Arc<Mutex<Client>>>,
    pub server: Option<Arc<Mutex<Server>>>,
    pub heartbeat_interval: Duration,
    pub peer_addresses: HashMap<String, String>,
}

impl PartialEq for AgentNet {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.signer == other.signer
            && self.verifiers == other.verifiers
            && self.addr == other.addr
            && self.heartbeat_interval == other.heartbeat_interval
            && self.peer_addresses == other.peer_addresses
    }
}

impl fmt::Debug for AgentNet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AgentNet")
            .field("id", &self.id)
            .field("signer", &self.signer)
            .field("verifiers", &self.verifiers)
            .field("addr", &self.addr)
            .field("heartbeat_interval", &self.heartbeat_interval)
            .field("peer_addresses", &self.peer_addresses)
            .field("clients", &"<Hidden>")
            .field("server", &"<Hidden>")
            .finish()
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct RemoteAgent {
    pub id: Cow<'static, str>,
    pub network: Arc<AgentNet>,
}

#[async_trait]
impl Collaborate for RemoteAgent {
    async fn handle_task(&self, task: Task) -> Result<()> {
        let msg = AgentMessage::Task(task);

        let mut message = Message {
            from: self.network.id.clone().into(),
            to: self.id.clone().into(),
            msg_type: MessageType::DelegateTask.into(),
            payload_json: serde_json::to_string(&msg)?,
            ..Default::default()
        };

        message.sign(&self.network.signer)?;

        if let Some(client) = self.network.clients.get(self.id.as_ref()) {
            client.lock().await.send(message).await?;
        } else {
            anyhow::bail!("No client found for remote agent id: {}", self.id);
        }

        Ok(())
    }

    async fn receive_message(&self, _: AgentMessage) -> Result<()> {
        // TODO: implement this func
        Ok(())
    }

    fn get_id(&self) -> &str {
        &self.id
    }
}

#[derive(Default, Clone, Debug)]
pub struct Swarm {
    pub locals: HashMap<String, Collaborator>,
    pub remotes: HashMap<String, Collaborator>,
    pub cap_index: HashMap<Capability, VecDeque<String>>,
    pub rr_idx: usize,
}

impl PartialEq for Swarm {
    fn eq(&self, other: &Self) -> bool {
        self.cap_index == other.cap_index && self.rr_idx == other.rr_idx
    }
}

impl Swarm {
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
            remotes: HashMap::new(),
            cap_index: HashMap::new(),
            rr_idx: 0,
        }
    }

    pub async fn register_local(&mut self, coll: Collaborator, caps: Vec<Capability>) {
        let id = coll.id().await.to_string();
        self.locals.insert(id.clone(), coll);
        for cap in caps {
            self.cap_index
                .entry(cap.clone())
                .or_default()
                .push_back(id.to_string());
        }
    }

    pub fn register_remote(
        &mut self,
        id: Cow<'static, str>,
        caps: Vec<Capability>,
        net: Arc<AgentNet>,
    ) {
        let proxy = Collaborator::Remote(RemoteAgent {
            id: id.clone(),
            network: net,
        });
        self.remotes.insert(id.to_string(), proxy.clone());
        for cap in caps {
            self.cap_index
                .entry(cap.clone())
                .or_default()
                .push_back(id.to_string());
        }
    }

    pub async fn assign_task_lb(&mut self, cap: &Capability, task: Task) -> Result<()> {
        let queue = self
            .cap_index
            .get_mut(cap)
            .ok_or_else(|| anyhow!("No agent has {:?}", cap))?;
        let id = queue[self.rr_idx % queue.len()].clone();
        self.rr_idx += 1;
        let coll = self.locals.get(&id).or(self.remotes.get(&id)).unwrap();
        delegate_task(coll, task).await
    }
}
