use crate::common::utils::{Communication, Status};
use crate::traits::agent::Agent;
use std::borrow::Cow;

/// Represents an agent with specific characteristics.
#[derive(Debug, PartialEq, Default, Clone)]
pub struct AgentKevin {
    /// The objective of the agent.
    objective: Cow<'static, str>,
    /// The position of the agent.
    position: Cow<'static, str>,
    /// The current status of the agent.
    status: Status,
    /// Memory containing exchanged communications.
    memory: Vec<Communication>,
}

impl AgentKevin {
    /// Adds a communication to the memory of the agent.
    ///
    /// # Arguments
    ///
    /// * `communication` - The communication to be added to the memory.
    pub fn add_communication(&mut self, communication: Communication) {
        self.memory.push(communication);
    }

    /// Creates a new instance of `AgentKevin` with owned strings.
    ///
    /// # Arguments
    ///
    /// * `objective` - The objective of the agent.
    /// * `position` - The position of the agent.
    ///
    /// # Returns
    ///
    /// A new instance of `AgentKevin`.
    pub fn new_owned(objective: String, position: String) -> Self {
        Self {
            objective: Cow::Owned(objective),
            position: Cow::Owned(position),
            status: Default::default(),
            memory: Default::default(),
        }
    }

    /// Creates a new instance of `AgentKevin` with borrowed strings.
    ///
    /// # Arguments
    ///
    /// * `objective` - The objective of the agent.
    /// * `position` - The position of the agent.
    ///
    /// # Returns
    ///
    /// A new instance of `AgentKevin`.
    pub fn new_borrowed(objective: &'static str, position: &'static str) -> Self {
        Self {
            objective: Cow::Borrowed(objective),
            position: Cow::Borrowed(position),
            status: Default::default(),
            memory: Default::default(),
        }
    }
}

impl Agent for AgentKevin {
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
