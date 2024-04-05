//! # `Agent` trait.
//!
//! This trait defines basic functionalities for agents.
//!
//! # Examples
//!
//! ```rust
//! use autogpt::common::utils::{Communication, Status};
//! use autogpt::traits::agent::Agent;
//! use std::borrow::Cow;
//!
//! /// A simple agent implementation.
//! #[derive(Debug)]
//! struct SimpleAgent {
//!     objective: Cow<'static, str>,
//!     position: Cow<'static, str>,
//!     status: Status,
//!     memory: Vec<Communication>,
//! }
//!
//! impl Agent for SimpleAgent {
//!     fn new(objective: Cow<'static, str>, position: Cow<'static, str>) -> Self {
//!         SimpleAgent {
//!             objective,
//!             position,
//!             status: Status::Idle,
//!             memory: Vec::new(),
//!         }
//!     }
//!
//!     fn update(&mut self, status: Status) {
//!         self.status = status;
//!     }
//!
//!     fn objective(&self) -> &Cow<'static, str> {
//!         &self.objective
//!     }
//!
//!     fn position(&self) -> &Cow<'static, str> {
//!         &self.position
//!     }
//!
//!     fn status(&self) -> &Status {
//!         &self.status
//!     }
//!
//!     fn memory(&self) -> &Vec<Communication> {
//!         &self.memory
//!     }
//! }
//!

use crate::common::utils::{Communication, Status};
use std::borrow::Cow;
use std::fmt::Debug;

/// A trait defining basic functionalities for agents.
pub trait Agent: Debug {
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
