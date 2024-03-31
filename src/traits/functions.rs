use crate::agents::agent::AgentGPT;
use crate::common::utils::Tasks;
use anyhow::Result;

/// Trait defining special functions for agents.
pub trait Functions {
    /// Get attributes from an agent.
    ///
    /// # Returns
    ///
    /// A reference to the agent.
    fn get_agent(&self) -> &AgentGPT;

    /// Execute special functions for an agent.
    ///
    /// # Arguments
    ///
    /// * `tasks` - The tasks associated with the agent.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    #[allow(async_fn_in_trait)]
    async fn execute(&mut self, tasks: &mut Tasks) -> Result<()>;
}
