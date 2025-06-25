//! # `Functions` and `AsyncFunctions` traits.
//!
//! These traits define special functions for agents.
//!
//! # Examples
//!
//! ```rust
//! use autogpt::agents::agent::AgentGPT;
//! use autogpt::common::utils::Tasks;
//! use anyhow::Result;
//! use autogpt::traits::functions::Functions;
//! use autogpt::traits::functions::AsyncFunctions;
//! use autogpt::common::utils::Communication;
//! use autogpt::prelude::async_trait;
//! use std::borrow::Cow;
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
//! }
//!
//! #[async_trait]
//! impl AsyncFunctions for SpecialFunctions {
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
//!     async fn execute<'a>(
//!        &'a mut self,
//!        tasks: &'a mut Tasks,
//!        _execute: bool,
//!        _browse: bool,
//!        _max_tries: u64,
//!     ) -> Result<()> {
//!         Ok(())
//!     }
//!
//!     /// Saves a communication to long-term memory for the agent.
//!     ///
//!     /// # Arguments
//!     ///
//!     /// * `communication` - The communication to save, which contains the role and content.
//!     ///
//!     /// # Returns
//!     ///
//!     /// (`Result<()>`): Result indicating the success or failure of saving the communication.
//!     async fn save_ltm(&mut self, _communication: Communication) -> Result<()> {
//!         Ok(())
//!     }
//!
//!     /// Retrieves all communications stored in the agent's long-term memory.
//!     ///
//!     /// # Returns
//!     ///
//!     /// (`Result<Vec<Communication>>`): A result containing a vector of communications retrieved from the agent's long-term memory.
//!     async fn get_ltm(&self) -> Result<Vec<Communication>> {
//!         Ok(vec![
//!             Communication {
//!                 role: Cow::Borrowed("system"),
//!                 content: Cow::Borrowed("System initialized."),
//!             },
//!             Communication {
//!                 role: Cow::Borrowed("user"),
//!                 content: Cow::Borrowed("Hello, autogpt!"),
//!             },
//!         ])
//!     }
//!
//!     /// Retrieves the concatenated context of all communications in the agent's long-term memory.
//!     ///
//!     /// # Returns
//!     ///
//!     /// (`String`): A string containing the concatenated role and content of all communications stored in the agent's long-term memory.
//!     async fn ltm_context(&self) -> String {
//!         let comms = [
//!             Communication {
//!                 role: Cow::Borrowed("system"),
//!                 content: Cow::Borrowed("System initialized."),
//!             },
//!             Communication {
//!                 role: Cow::Borrowed("user"),
//!                 content: Cow::Borrowed("Hello, autogpt!"),
//!             },
//!         ];
//!
//!         comms
//!             .iter()
//!             .map(|c| format!("{}: {}", c.role, c.content))
//!             .collect::<Vec<_>>()
//!             .join("\n")
//!     }
//! }
//!

use crate::agents::agent::AgentGPT;
#[cfg(feature = "mem")]
use crate::common::utils::Communication;
use crate::common::utils::Tasks;
use anyhow::Result;
use async_trait::async_trait;

/// Trait to retrieve an agent.
pub trait Functions {
    /// Get attributes from an agent.
    ///
    /// # Returns
    ///
    /// A reference to the agent.
    fn get_agent(&self) -> &AgentGPT;
}

/// Trait defining special functions for agents.\
#[async_trait]
pub trait AsyncFunctions: Send + Sync {
    /// Execute special functions for an agent.
    ///
    /// # Arguments
    ///
    /// * `tasks` - The tasks associated with the agent.
    /// * `execute` - A boolean indicating whether to execute the generated code by the agent.
    /// * `browse` - Whether to open a browser.
    /// * `max_tries` - A integer indicating the max number of tries fixing code bugs.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    #[allow(async_fn_in_trait)]
    async fn execute<'a>(
        &'a mut self,
        tasks: &'a mut Tasks,
        execute: bool,
        browse: bool,
        max_tries: u64,
    ) -> Result<()>;

    /// Save a communication into long-term memory.
    ///
    /// # Arguments
    ///
    /// * `communication` - The communication to save.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    #[allow(async_fn_in_trait)]
    #[cfg(feature = "mem")]
    async fn save_ltm<'a>(&'a mut self, communication: Communication) -> Result<()>;

    /// Get the long-term memory of an agent.
    ///
    /// # Returns
    ///
    /// A result containing a vector of communications.
    #[allow(async_fn_in_trait)]
    #[cfg(feature = "mem")]
    async fn get_ltm<'a>(&'a self) -> Result<Vec<Communication>>;

    /// Retrieve the long-term memory context as a string.
    ///
    /// # Returns
    ///
    /// A string containing the concatenated context of the agent's memory.
    #[allow(async_fn_in_trait)]
    #[cfg(feature = "mem")]
    async fn ltm_context<'a>(&'a self) -> String;
}
