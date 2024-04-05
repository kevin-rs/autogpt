//! # `Functions` trait.
//!
//! This trait defines special functions for agents.
//!
//! # Examples
//!
//! ```rust
//! use autogpt::agents::agent::AgentGPT;
//! use autogpt::common::utils::Tasks;
//! use anyhow::Result;
//! use autogpt::traits::functions::Functions;
//!
//!
//! /// A struct implementing the `Functions` trait.
//! struct SpecialFunctions {
//!     agent: AgentGPT,
//! }
//!
//! impl SpecialFunctions {
//!     /// Creates a new instance of `SpecialFunctions`.
//!     ///
//!     /// # Arguments
//!     ///
//!     /// * `agent` - The agent to associate with the functions.
//!     fn new(agent: AgentGPT) -> Self {
//!         SpecialFunctions { agent }
//!     }
//! }
//!
//! impl Functions for SpecialFunctions {
//!     /// Get fields from an agent.
//!     ///
//!     /// # Returns
//!     ///
//!     /// A reference to the agent.
//!     fn get_agent(&self) -> &AgentGPT {
//!         &self.agent
//!     }
//!
//!     /// Execute special functions for an agent.
//!     ///
//!     /// # Arguments
//!     ///
//!     /// * `tasks` - The tasks associated with the agent.
//!     /// * `execute` - A boolean indicating whether to execute the generated code by the agent.
//!     /// * `max_tries` - A integer indicating the max number of tries fixing code bugs.
//!     ///
//!     /// # Returns
//!     ///
//!     /// A result indicating success or failure.
//!     async fn execute(&mut self, tasks: &mut Tasks, execute: bool, max_tries: u64) -> Result<()> {
//!         // Implementation here
//!         unimplemented!()
//!     }
//! }
//!

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
    /// * `execute` - A boolean indicating whether to execute the generated code by the agent.
    /// * `max_tries` - A integer indicating the max number of tries fixing code bugs.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    #[allow(async_fn_in_trait)]
    async fn execute(&mut self, tasks: &mut Tasks, execute: bool, max_tries: u64) -> Result<()>;
}
