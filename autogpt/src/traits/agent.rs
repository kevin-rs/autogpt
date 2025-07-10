//! # `Agent` trait.
//!
//! This trait defines basic functionalities for agents.
//!
//! # Examples
//!
//! ```rust
//! use autogpt::common::utils::{
//!     Capability, Communication, ContextManager, Knowledge, Persona, Planner,
//!     Reflection, Status, TaskScheduler, Tool, Task
//! };
//! use autogpt::collaboration::Collaborator;
//! use autogpt::traits::agent::Agent;
//! use autogpt::traits::composite::AgentFunctions;
//! use std::borrow::Cow;
//! use std::collections::HashSet;
//! use tokio::sync::Mutex;
//! use std::sync::Arc;
//!
//! /// A simple agent implementation that satisfies the full Agent trait.
//! #[derive(Debug)]
//! struct SimpleAgent {
//!     objective: Cow<'static, str>,
//!     position: Cow<'static, str>,
//!     status: Status,
//!     memory: Vec<Communication>,
//!     tools: Vec<Tool>,
//!     knowledge: Knowledge,
//!     planner: Option<Planner>,
//!     persona: Persona,
//!     collaborators: Vec<Collaborator>,
//!     reflection: Option<Reflection>,
//!     scheduler: Option<TaskScheduler>,
//!     capabilities: HashSet<Capability>,
//!     context: ContextManager,
//!     tasks: Vec<Task>,
//! }
//!
//! impl Agent for SimpleAgent {
//!     fn new(objective: Cow<'static, str>, position: Cow<'static, str>) -> Self {
//!         SimpleAgent {
//!             objective,
//!             position,
//!             status: Status::Idle,
//!             memory: vec![],
//!             tools: vec![],
//!             knowledge: Knowledge::default(),
//!             planner: None,
//!             persona: Persona {
//!                 name: Cow::Borrowed("Default"),
//!                 traits: vec![],
//!                 behavior_script: None,
//!             },
//!             collaborators: vec![],
//!             reflection: None,
//!             scheduler: None,
//!             capabilities: HashSet::new(),
//!             context: ContextManager {
//!                 recent_messages: vec![],
//!                 focus_topics: vec![],
//!             },
//!             tasks: vec![],
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
//!
//!     fn tools(&self) -> &Vec<Tool> {
//!         &self.tools
//!     }
//!
//!     fn knowledge(&self) -> &Knowledge {
//!         &self.knowledge
//!     }
//!
//!     fn planner(&self) -> Option<&Planner> {
//!         self.planner.as_ref()
//!     }
//!
//!     fn persona(&self) -> &Persona {
//!         &self.persona
//!     }
//!
//!     fn collaborators(&self) -> Vec<Collaborator> {
//!         let mut all = Vec::new();
//!         all.extend(self.agent.local_collaborators.values().cloned());
//!         all.extend(self.agent.remote_collaborators.values().cloned());
//!         all
//!     }
//!
//!     fn reflection(&self) -> Option<&Reflection> {
//!         self.reflection.as_ref()
//!     }
//!
//!     fn scheduler(&self) -> Option<&TaskScheduler> {
//!         self.scheduler.as_ref()
//!     }
//!
//!     fn capabilities(&self) -> &HashSet<Capability> {
//!         &self.capabilities
//!     }
//!
//!     fn context(&self) -> &ContextManager {
//!         &self.context
//!     }
//!
//!     fn tasks(&self) -> &Vec<Task> {
//!         &self.tasks
//!     }
//!
//!     fn memory_mut(&mut self) -> &mut Vec<Communication> {
//!         &mut self.memory
//!     }
//!
//!     fn planner_mut(&mut self) -> Option<&mut Planner> {
//!         self.planner.as_mut()
//!     }
//!
//!     fn context_mut(&mut self) -> &mut ContextManager {
//!         &mut self.context
//!     }
//! }
//! ```
//!

#[cfg(feature = "net")]
use crate::collaboration::Collaborator;
use crate::common::utils::{
    Capability, Communication, ContextManager, Knowledge, Persona, Planner, Reflection, Status,
    Task, TaskScheduler, Tool,
};
use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt::Debug;

/// A trait defining basic functionalities for agents.
pub trait Agent: Debug {
    /// Creates a new instance of an agent.
    ///
    /// # Arguments
    ///
    /// * `objective` - The objective of the agent.
    /// * `position` - The position of the agent.
    fn new(objective: Cow<'static, str>, position: Cow<'static, str>) -> Self
    where
        Self: Sized;

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

    /// Returns the agent's tools
    fn tools(&self) -> &Vec<Tool>;

    /// Returns the knowledge base
    fn knowledge(&self) -> &Knowledge;

    /// Returns a mutable reference to the planner (if any)
    fn planner(&self) -> Option<&Planner>;

    /// Returns the agent's persona
    fn persona(&self) -> &Persona;

    /// Returns a list of local and remote collaborators agents
    #[cfg(feature = "net")]
    fn collaborators(&self) -> Vec<Collaborator>;

    /// Returns optional self-reflection module
    fn reflection(&self) -> Option<&Reflection>;

    /// Returns optional task scheduler
    fn scheduler(&self) -> Option<&TaskScheduler>;

    /// Returns the agent's capabilities
    fn capabilities(&self) -> &HashSet<Capability>;

    /// Returns the agent's context manager
    fn context(&self) -> &ContextManager;

    /// Returns the current list of tasks
    fn tasks(&self) -> &Vec<Task>;

    /// Mutable access to memory (messages)
    fn memory_mut(&mut self) -> &mut Vec<Communication>;

    /// Mutable access to planner (if any)
    fn planner_mut(&mut self) -> Option<&mut Planner>;

    /// Mutable access to context manager
    fn context_mut(&mut self) -> &mut ContextManager;
}
