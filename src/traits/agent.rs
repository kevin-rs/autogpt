use crate::common::utils::{Communication, Status};
use std::borrow::Cow;

/// A trait defining basic functionalities for agents.
pub trait Agent {
    /// Creates a new instance of an agent.
    ///
    /// # Arguments
    ///
    /// * `objective` - The objective of the agent.
    /// * `position` - The position of the agent.
    fn new(objective: Cow<'static, str>, position: Cow<'static, str>) -> Self;

    /// Updates the status of the agent.
    ///
    /// # Arguments
    ///
    /// * `status` - The new status to be assigned to the agent.
    fn update(&mut self, status: Status);

    /// Retrieves the objective of the agent.
    fn objective(&self) -> &Cow<'static, str>;

    /// Retrieves the position of the agent.
    fn position(&self) -> &Cow<'static, str>;

    /// Retrieves the current status of the agent.
    fn status(&self) -> &Status;

    /// Retrieves the memory of the agent containing exchanged messages.
    fn memory(&self) -> &Vec<Communication>;
}
