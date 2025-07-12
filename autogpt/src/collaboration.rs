use crate::common::utils::{AgentMessage, Task};
use crate::traits::functions::Collaborate;
use anyhow::Result;
use async_trait::async_trait;
use iac_rs::prelude::*;
use serde_json;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
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

pub async fn delegate_task(collab: Collaborator, task: Task) -> Result<()> {
    match collab {
        Collaborator::Local(agent) => agent.lock().await.handle_task(task).await,
        Collaborator::Remote(mut agent) => agent.handle_task(task).await,
    }
}

#[derive(Debug, Clone)]
pub struct RemoteAgent {
    pub id: Cow<'static, str>,
    pub signer: Signer,
    pub clients: HashMap<String, Arc<Mutex<Client>>>,
}

#[async_trait]
impl Collaborate for RemoteAgent {
    async fn handle_task(&mut self, task: Task) -> Result<()> {
        let msg = AgentMessage::Task(task);

        let mut message = Message {
            from: "AgentGPT".into(),
            to: self.id.clone().into(),
            msg_type: MessageType::DelegateTask,
            payload_json: serde_json::to_string(&msg)?,
            ..Default::default()
        };

        message.sign(&self.signer)?;

        if let Some(client) = self.clients.get(self.id.as_ref()) {
            client.lock().await.send(message).await?;
        } else {
            anyhow::bail!("No client found for remote agent id: {}", self.id);
        }

        Ok(())
    }

    async fn receive_message(&mut self, _msg: AgentMessage) -> Result<()> {
        Ok(())
    }

    fn get_id(&self) -> &str {
        &self.id
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
