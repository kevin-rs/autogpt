#![doc = include_str!("../INSTALLATION.md")]

#[cfg(any(feature = "oai", feature = "gem", feature = "cld", feature = "xai"))]
use {futures::future::join_all, tokio::task, tracing::error};

#[cfg(feature = "img")]
pub use crate::agents::designer::DesignerGPT;
#[cfg(feature = "git")]
pub use crate::agents::git::GitGPT;
#[cfg(feature = "mail")]
pub use crate::agents::mailer::MailerGPT;

#[cfg(feature = "gpt")]
pub use {
    crate::agents::architect::ArchitectGPT, crate::agents::backend::BackendGPT,
    crate::agents::frontend::FrontendGPT, crate::agents::manager::ManagerGPT,
    crate::agents::optimizer::OptimizerGPT,
};

#[allow(unused)]
pub use {
    crate::agents,
    crate::agents::agent::AgentGPT,
    crate::common::utils::{
        AgentMessage, Capability, ClientType, Communication, ContextManager, Knowledge, Persona,
        Planner, Reflection, Scope, Status, Task, TaskScheduler, Tool,
    },
    crate::traits::agent::Agent,
    crate::traits::composite::AgentFunctions,
    crate::traits::functions::{AsyncFunctions, Collaborate, Executor, Functions, ReqResponse},
    anyhow::{Result, anyhow},
    async_trait::async_trait,
    auto_derive::Auto,
    std::collections::HashSet,
    std::{borrow::Cow, sync::Arc},
    tokio::sync::Mutex,
    uuid::Uuid,
};

#[cfg(feature = "net")]
pub use {
    crate::collaboration::Collaborator, iac_rs::prelude::Message as IacMessage, iac_rs::prelude::*,
};

#[cfg(not(feature = "net"))]
#[allow(unused_imports)]
use tracing::debug;

#[cfg(feature = "mem")]
pub use {
    crate::common::memory::load_long_term_memory, crate::common::memory::long_term_memory_context,
    crate::common::memory::save_long_term_memory,
};

#[cfg(feature = "oai")]
pub use {openai_dive::v1::models::FlagshipModel, openai_dive::v1::resources::chat::*};

#[cfg(feature = "cld")]
pub use anthropic_ai_sdk::types::message::{
    ContentBlock, CreateMessageParams, Message as AnthMessage, MessageClient,
    RequiredMessageParams, Role,
};

#[cfg(feature = "gem")]
pub use gems::{
    chat::ChatBuilder,
    imagen::ImageGenBuilder,
    messages::{Content, Message},
    models::Model,
    stream::StreamBuilder,
    traits::CTrait,
};

#[cfg(feature = "xai")]
pub use x_ai::{
    chat_compl::{ChatCompletionsRequestBuilder, Message as XaiMessage},
    traits::ChatCompletionsFetcher,
};

#[cfg(feature = "oai")]
pub use openai_dive;

#[cfg(feature = "gem")]
pub use gems;

#[cfg(feature = "xai")]
pub use x_ai;

#[allow(unreachable_code)]
/// Represents an AutoGPT instance managing multiple agents and their execution settings.
pub struct AutoGPT {
    /// Unique identifier for this AutoGPT instance.
    pub id: Uuid,

    /// Collection of GPT agents. These agents run concurrently to handle assigned tasks.
    pub agents: Vec<Arc<Mutex<Box<dyn AgentFunctions>>>>,

    /// Flag indicating whether agents should execute their tasks.
    /// Typically set to `true` to enable execution.
    pub execute: bool,

    /// Flag indicating whether agents are allowed to browse external resources during execution.
    /// Set to `true` to enable browsing capabilities.
    pub browse: bool,

    /// Maximum number of retry attempts an agent should make upon task execution failure.
    pub max_tries: u64,

    /// Scope permission: whether agents have CRUD access.
    /// `true` enables CRUD operations within the task scope.
    pub crud: bool,

    /// Scope permission: whether agents have authorization capabilities.
    /// `true` allows agents to perform authorization-related actions.
    pub auth: bool,

    /// Scope permission: whether agents can access external resources or services.
    /// `true` grants permission to interact with external endpoints.
    pub external: bool,
}

impl Default for AutoGPT {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            agents: vec![],
            execute: true,
            browse: false,
            max_tries: 1,
            crud: true,
            auth: false,
            external: true,
        }
    }
}
impl AutoGPT {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn execute(mut self, execute: bool) -> Self {
        self.execute = execute;
        self
    }

    pub fn browse(mut self, browse: bool) -> Self {
        self.browse = browse;
        self
    }

    pub fn max_tries(mut self, max_tries: u64) -> Self {
        self.max_tries = max_tries;
        self
    }
    pub fn crud(mut self, enabled: bool) -> Self {
        self.crud = enabled;
        self
    }

    pub fn auth(mut self, enabled: bool) -> Self {
        self.auth = enabled;
        self
    }

    pub fn external(mut self, enabled: bool) -> Self {
        self.external = enabled;
        self
    }

    #[cfg(any(feature = "oai", feature = "gem", feature = "cld", feature = "xai"))]
    pub fn with<A>(mut self, agents: A) -> Self
    where
        A: Into<Vec<Arc<Mutex<Box<dyn AgentFunctions>>>>>,
    {
        self.agents = agents.into();
        self
    }

    #[cfg(any(feature = "oai", feature = "gem", feature = "cld", feature = "xai"))]
    pub fn build(self) -> Result<Self> {
        Ok(Self {
            id: self.id,
            agents: self.agents,
            execute: self.execute,
            browse: self.browse,
            max_tries: self.max_tries,
            crud: self.crud,
            auth: self.auth,
            external: self.external,
        })
    }

    #[cfg(any(feature = "oai", feature = "gem", feature = "cld", feature = "xai"))]
    pub async fn run(&self) -> Result<String> {
        if self.agents.is_empty() {
            return Err(anyhow!("No agents to run."));
        }

        let mut handles = Vec::with_capacity(self.agents.len());

        let execute = self.execute;
        let browse = self.browse;
        let max_tries = self.max_tries;
        let crud = self.crud;
        let auth = self.auth;
        let external = self.external;

        for (i, agent_arc) in self.agents.iter().cloned().enumerate() {
            let agent_clone = Arc::clone(&agent_arc);
            let agent_objective = agent_arc.lock().await.get_agent().objective().clone();

            let tasks = Arc::new(Mutex::new(Task {
                description: agent_objective.clone(),
                scope: Some(Scope {
                    crud,
                    auth,
                    external,
                }),
                urls: None,
                frontend_code: None,
                backend_code: None,
                api_schema: None,
            }));

            let tasks_clone = Arc::clone(&tasks);

            let handle = task::spawn(async move {
                let mut locked_tasks = tasks_clone.lock().await;
                let mut agent = agent_clone.lock().await;

                match agent
                    .execute(&mut locked_tasks, execute, browse, max_tries)
                    .await
                {
                    Ok(_) => {
                        debug!("Agent {} ({}) executed successfully", i, agent_objective);
                        Ok::<(), anyhow::Error>(())
                    }
                    Err(err) => {
                        error!(
                            "Agent {} ({}) failed with error: {}",
                            i, agent_objective, err
                        );
                        Err(anyhow!("Agent {} failed: {}", i, err))
                    }
                }
            });

            handles.push(handle);
        }

        let results = join_all(handles).await;

        let failures: Vec<_> = results
            .into_iter()
            .enumerate()
            .filter_map(|(i, res)| match res {
                Ok(Err(e)) => Some(format!("Agent {i}: {e}")),
                Err(join_err) => Some(format!("Agent {i} panicked: {join_err}")),
                _ => None,
            })
            .collect();

        if !failures.is_empty() {
            return Err(anyhow!("Some agents failed:\n{}", failures.join("\n")));
        }

        Ok("All agents executed successfully.".to_string())
    }
}
