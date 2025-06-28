#![doc = include_str!("../INSTALLATION.md")]

#[cfg(any(feature = "oai", feature = "gem", feature = "cld"))]
use {
    futures::future::join_all,
    tokio::task,
    tracing::{debug, error},
};

#[cfg(feature = "img")]
pub use crate::agents::designer::DesignerGPT;
#[cfg(feature = "git")]
pub use crate::agents::git::GitGPT;
#[cfg(feature = "mail")]
pub use crate::agents::mailer::MailerGPT;

#[allow(unused)]
pub use {
    crate::agents,
    crate::agents::agent::AgentGPT,
    crate::agents::architect::ArchitectGPT,
    crate::agents::backend::BackendGPT,
    crate::agents::frontend::FrontendGPT,
    crate::agents::manager::ManagerGPT,
    crate::agents::optimizer::OptimizerGPT,
    crate::common::utils::{
        Capability, ClientType, Communication, ContextManager, Knowledge, Persona, Planner,
        Reflection, Scope, Status, Task, TaskScheduler, Tool,
    },
    crate::traits::agent::Agent,
    crate::traits::composite::AgentFunctions,
    crate::traits::functions::{AgentExecutor, AsyncFunctions, Functions},
    anyhow::{Result, anyhow},
    async_trait::async_trait,
    auto_derive::Auto,
    std::collections::HashSet,
    std::{borrow::Cow, sync::Arc},
    tokio::sync::Mutex,
    uuid::Uuid,
};

#[cfg(feature = "mem")]
pub use {
    crate::common::memory::load_long_term_memory, crate::common::memory::long_term_memory_context,
    crate::common::memory::save_long_term_memory,
};

#[cfg(feature = "oai")]
pub use openai_dive;

#[cfg(feature = "gem")]
pub use gems;

#[cfg(feature = "xai")]
pub use x_ai;

#[derive(Default)]
#[allow(unreachable_code)]
pub struct AutoGPT {
    /// Unique identifier for AutoGPT instance.
    pub id: Uuid,
    /// Represents GPT agents responsible for handling tasks in parallel.
    pub agents: Vec<Arc<Mutex<Box<dyn AgentFunctions>>>>,
}

impl AutoGPT {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            agents: vec![],
        }
    }

    pub fn id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    #[cfg(any(feature = "oai", feature = "gem", feature = "cld"))]
    pub fn with<A>(mut self, agents: A) -> Self
    where
        A: Into<Vec<Arc<Mutex<Box<dyn AgentFunctions>>>>>,
    {
        self.agents = agents.into();
        self
    }

    #[cfg(any(feature = "oai", feature = "gem", feature = "cld"))]
    pub fn build(self) -> Result<Self> {
        Ok(Self {
            id: self.id,
            agents: self.agents,
        })
    }

    #[cfg(any(feature = "oai", feature = "gem", feature = "cld"))]
    pub async fn run(&self) -> Result<String> {
        if self.agents.is_empty() {
            return Err(anyhow!("No agents to run."));
        }

        let mut handles = Vec::with_capacity(self.agents.len());

        for (i, agent_arc) in self.agents.iter().cloned().enumerate() {
            let agent_clone = Arc::clone(&agent_arc);
            let agent_objective = agent_arc.lock().await.get_agent().objective().clone();

            let tasks = Arc::new(Mutex::new(Task {
                description: agent_objective.clone(),
                scope: Some(Scope {
                    crud: true,
                    auth: false,
                    external: true,
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

                match agent.execute(&mut locked_tasks, true, false, 3).await {
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
                Ok(Err(e)) => Some(format!("Agent {}: {}", i, e)),
                Err(join_err) => Some(format!("Agent {} panicked: {}", i, join_err)),
                _ => None,
            })
            .collect();

        if !failures.is_empty() {
            return Err(anyhow!("Some agents failed:\n{}", failures.join("\n")));
        }

        Ok("All agents executed successfully.".to_string())
    }
}
