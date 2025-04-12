use crate::agents::architect::ArchitectGPT;
use crate::agents::backend::BackendGPT;
#[cfg(feature = "img")]
use crate::agents::designer::DesignerGPT;
use crate::agents::frontend::FrontendGPT;
use crate::agents::git::GitGPT;
use crate::common::utils::Tasks;
use crate::traits::agent::Agent;
use crate::traits::functions::Functions;
use anyhow::Result;

/// Enum representing different types of GPT agents.
#[derive(Debug, Clone)]
pub enum AgentType {
    /// Architect GPT agent.
    Architect(ArchitectGPT),
    /// Backend GPT agent.
    Backend(BackendGPT),
    /// Frontend GPT agent.
    Frontend(FrontendGPT),
    /// Designer GPT agent.
    #[cfg(feature = "img")]
    Designer(DesignerGPT),
    /// Git GPT agent.
    #[cfg(feature = "git")]
    Git(GitGPT),
}

impl AgentType {
    /// Asynchronously executes tasks associated with the agent.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to tasks to be executed.
    /// * `execute` - A boolean indicating whether to execute the tasks.
    /// * `max_tries` - Maximum number of attempts to execute tasks.
    ///
    /// # Returns
    ///
    /// (`Result<()>`): Result indicating success or failure of task execution.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in executing tasks.
    ///
    /// # Business Logic
    ///
    /// - Executes tasks associated with the agent based on its type.
    ///
    pub async fn execute(
        &mut self,
        tasks: &mut Tasks,
        execute: bool,
        browse: bool,
        max_tries: u64,
    ) -> Result<()> {
        match self {
            AgentType::Architect(agent) => agent.execute(tasks, execute, browse, max_tries).await,
            AgentType::Backend(agent) => agent.execute(tasks, execute, browse, max_tries).await,
            AgentType::Frontend(agent) => agent.execute(tasks, execute, browse, max_tries).await,
            #[cfg(feature = "img")]
            AgentType::Designer(agent) => agent.execute(tasks, execute, browse, max_tries).await,
            #[cfg(feature = "git")]
            AgentType::Git(agent) => agent.execute(tasks, execute, browse, max_tries).await,
        }
    }

    /// Retrieves the position of the agent.
    ///
    /// # Returns
    ///
    /// (`String`): The position of the agent.
    ///
    /// # Business Logic
    ///
    /// - Retrieves the position of the agent based on its type.
    ///
    pub fn position(&self) -> String {
        match self {
            AgentType::Architect(agent) => agent.get_agent().position().to_string(),
            AgentType::Backend(agent) => agent.get_agent().position().to_string(),
            AgentType::Frontend(agent) => agent.get_agent().position().to_string(),
            AgentType::Git(agent) => agent.get_agent().position().to_string(),
            _ => "Any".to_string(),
        }
    }
}
