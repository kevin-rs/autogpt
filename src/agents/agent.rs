//! # `AgentGPT` agent.
//!

use crate::common::utils::{Communication, Status};
use crate::traits::agent::Agent;
use std::borrow::Cow;
use uuid::Uuid;

/// Represents an agent with specific characteristics.
#[derive(Debug, PartialEq, Clone)]
pub struct AgentGPT {
    /// Unique identifier for the agent.
    pub id: Cow<'static, str>,
    /// The objective of the agent.
    pub objective: Cow<'static, str>,
    /// The position of the agent.
    pub position: Cow<'static, str>,
    /// The current status of the agent.
    pub status: Status,
    /// Hot memory containing exchanged communications between agents and/or user.
    pub memory: Vec<Communication>,
}

impl Default for AgentGPT {
    fn default() -> Self {
        AgentGPT {
            id: Cow::Owned(Uuid::new_v4().to_string()),
            objective: Cow::Borrowed(""),
            position: Cow::Borrowed(""),
            status: Status::default(),
            memory: vec![],
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
    /// A new instance of `AgentGPT`.
    pub fn new_owned(objective: String, position: String) -> Self {
        Self {
            id: Cow::Owned(Uuid::new_v4().to_string()),
            objective: Cow::Owned(objective),
            position: Cow::Owned(position),
            status: Default::default(),
            memory: Default::default(),
        }
    }

    /// Creates a new instance of `AgentGPT` with borrowed strings.
    ///
    /// # Arguments
    ///
    /// * `objective` - The objective of the agent.
    /// * `position` - The position of the agent.
    ///
    /// # Returns
    ///
    /// A new instance of `AgentGPT`.
    pub fn new_borrowed(objective: &'static str, position: &'static str) -> Self {
        Self {
            id: Cow::Owned(Uuid::new_v4().to_string()),
            objective: Cow::Borrowed(objective),
            position: Cow::Borrowed(position),
            status: Default::default(),
            memory: Default::default(),
        }
    }
}

impl Agent for AgentGPT {
    /// Creates a new instance of an agent with the specified objective and position.
    ///
    /// # Arguments
    ///
    /// * `objective` - The objective of the agent.
    /// * `position` - The position of the agent.
    ///
    /// # Returns
    ///
    /// A new instance of the `Agent` struct.
    fn new(objective: Cow<'static, str>, position: Cow<'static, str>) -> Self {
        Self {
            id: Cow::Owned(Uuid::new_v4().to_string()),
            objective,
            position,
            status: Default::default(),
            memory: Default::default(),
        }
    }

    /// Updates the status of the agent.
    ///
    /// # Arguments
    ///
    /// * `status` - The new status to be assigned to the agent.
    fn update(&mut self, status: Status) {
        self.status = status;
    }

    /// Retrieves the objective of the agent.
    ///
    /// # Returns
    ///
    /// A reference to the objective of the agent.
    fn objective(&self) -> &Cow<'static, str> {
        &self.objective
    }

    /// Retrieves the position of the agent.
    ///
    /// # Returns
    ///
    /// A reference to the position of the agent.
    fn position(&self) -> &Cow<'static, str> {
        &self.position
    }

    /// Retrieves the current status of the agent.
    ///
    /// # Returns
    ///
    /// A reference to the current status of the agent.
    fn status(&self) -> &Status {
        &self.status
    }

    /// Retrieves the memory of the agent containing exchanged communications.
    ///
    /// # Returns
    ///
    /// A reference to the memory of the agent.
    fn memory(&self) -> &Vec<Communication> {
        &self.memory
    }
}
