//! # `AgentGPT` agent.
//!

use crate::common::utils::{
    Capability, Communication, ContextManager, Knowledge, Persona, Planner, Reflection, Status,
    Task, TaskScheduler, Tool, default_eval_fn,
};
use crate::traits::agent::Agent;
use crate::traits::composite::AgentFunctions;
use derivative::Derivative;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Represents an agent with memory, tools, and other autonomous capabilities.
#[derive(Derivative)]
#[derivative(PartialEq, Debug, Clone)]
pub struct AgentGPT {
    /// Unique identifier for the agent.
    pub id: Cow<'static, str>,

    /// The objective or mission of the agent.
    pub objective: Cow<'static, str>,

    /// The logical or physical position of the agent.
    pub position: Cow<'static, str>,

    /// The current operational status of the agent.
    pub status: Status,

    /// Hot memory containing past communications.
    pub memory: Vec<Communication>,

    /// Tools available to the agent.
    pub tools: Vec<Tool>,

    /// Structured knowledge base used for reasoning or retrieval.
    pub knowledge: Knowledge,

    /// Optional planner to manage goal sequencing.
    pub planner: Option<Planner>,

    /// Persona defines behavior style and traits.
    pub persona: Persona,

    /// Other agents this agent collaborates with.
    #[derivative(PartialEq = "ignore")]
    pub collaborators: Vec<Arc<Mutex<Box<dyn AgentFunctions>>>>,

    /// Optional self-reflection module for introspection or evaluation.
    pub reflection: Option<Reflection>,

    /// Optional task scheduler for time-based goal management.
    pub scheduler: Option<TaskScheduler>,

    /// Capabilities this agent has access to (e.g. CodeGen, WebSearch).
    pub capabilities: HashSet<Capability>,

    /// Manages context for conversation and topic focus.
    pub context: ContextManager,

    /// List of tasks assigned to this agent.
    pub tasks: Vec<Task>,
}

impl Default for AgentGPT {
    fn default() -> Self {
        Self {
            id: Cow::Owned(Uuid::new_v4().to_string()),
            objective: Cow::Borrowed(""),
            position: Cow::Borrowed(""),
            status: Status::default(),
            memory: vec![],
            tools: vec![],
            knowledge: Knowledge::default(),
            planner: None,
            persona: Persona {
                name: Cow::Borrowed("Default"),
                traits: vec![],
                behavior_script: None,
            },
            collaborators: vec![],
            reflection: None,
            scheduler: None,
            capabilities: HashSet::new(),
            context: ContextManager {
                recent_messages: vec![],
                focus_topics: vec![],
            },
            tasks: vec![],
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
    /// A new fully initialized instance of `AgentGPT`.
    pub fn new_owned(objective: String, position: String) -> Self {
        Self {
            id: Cow::Owned(Uuid::new_v4().to_string()),
            objective: Cow::Owned(objective),
            position: Cow::Owned(position.clone()),
            status: Status::Idle,

            memory: vec![],

            tools: vec![],

            knowledge: Knowledge {
                facts: HashMap::default(),
            },

            planner: Some(Planner {
                current_plan: vec![],
            }),

            persona: Persona {
                name: position.into(),
                traits: vec![],
                behavior_script: None,
            },

            collaborators: vec![],

            reflection: Some(Reflection {
                recent_logs: vec![],
                evaluation_fn: default_eval_fn,
            }),

            scheduler: Some(TaskScheduler {
                scheduled_tasks: vec![],
            }),

            capabilities: HashSet::default(),

            context: ContextManager {
                recent_messages: vec![],
                focus_topics: vec![],
            },

            tasks: vec![],
        }
    }

    /// Creates a new instance of `AgentGPT` with borrowed string slices.
    ///
    /// # Arguments
    ///
    /// * `objective` - The objective of the agent.
    /// * `position` - The position of the agent.
    ///
    /// # Returns
    ///
    /// A new fully initialized instance of `AgentGPT`.
    pub fn new_borrowed(objective: &'static str, position: &'static str) -> Self {
        Self {
            id: Cow::Owned(Uuid::new_v4().to_string()),
            objective: Cow::Borrowed(objective),
            position: Cow::Borrowed(position),
            status: Status::Idle,

            memory: vec![],

            tools: vec![],

            knowledge: Knowledge {
                facts: HashMap::default(),
            },

            planner: Some(Planner {
                current_plan: vec![],
            }),

            persona: Persona {
                name: position.into(),
                traits: vec![],
                behavior_script: None,
            },

            collaborators: vec![],

            reflection: Some(Reflection {
                recent_logs: vec![],
                evaluation_fn: default_eval_fn,
            }),

            scheduler: Some(TaskScheduler {
                scheduled_tasks: vec![],
            }),

            capabilities: HashSet::default(),

            context: ContextManager {
                recent_messages: vec![],
                focus_topics: vec![],
            },

            tasks: vec![],
        }
    }
}

impl Agent for AgentGPT {
    /// Creates a new `AgentGPT` instance with the given objective and position.
    fn new(objective: Cow<'static, str>, position: Cow<'static, str>) -> Self {
        Self {
            id: Cow::Owned(Uuid::new_v4().to_string()),

            objective,
            position: position.clone(),
            status: Status::Idle,

            memory: vec![],

            tools: vec![],

            knowledge: Knowledge {
                facts: HashMap::default(),
            },

            planner: Some(Planner {
                current_plan: vec![],
            }),

            persona: Persona {
                name: position,
                traits: vec![],
                behavior_script: None,
            },

            collaborators: vec![],

            reflection: Some(Reflection {
                recent_logs: vec![],
                evaluation_fn: default_eval_fn,
            }),

            scheduler: Some(TaskScheduler {
                scheduled_tasks: vec![],
            }),

            capabilities: HashSet::default(),

            context: ContextManager {
                recent_messages: vec![],
                focus_topics: vec![],
            },

            tasks: vec![],
        }
    }

    /// Updates the agent's operational status.
    fn update(&mut self, status: Status) {
        self.status = status;
    }

    /// Returns the agent's objective.
    fn objective(&self) -> &Cow<'static, str> {
        &self.objective
    }

    /// Returns the agent's current position.
    fn position(&self) -> &Cow<'static, str> {
        &self.position
    }

    /// Returns the agent's current status.
    fn status(&self) -> &Status {
        &self.status
    }

    /// Returns the agent's memory log of communications.
    fn memory(&self) -> &Vec<Communication> {
        &self.memory
    }

    /// Returns the agent's available tools.
    fn tools(&self) -> &Vec<Tool> {
        &self.tools
    }

    /// Returns the agent's structured knowledge base.
    fn knowledge(&self) -> &Knowledge {
        &self.knowledge
    }

    /// Returns an optional reference to the agent's planner.
    fn planner(&self) -> Option<&Planner> {
        self.planner.as_ref()
    }

    /// Returns the agent's persona configuration.
    fn persona(&self) -> &Persona {
        &self.persona
    }

    /// Returns a list of agents this agent collaborates with.
    fn collaborators(&self) -> &Vec<Arc<Mutex<Box<dyn AgentFunctions>>>> {
        &self.collaborators
    }

    /// Returns an optional reference to the self-reflection module.
    fn reflection(&self) -> Option<&Reflection> {
        self.reflection.as_ref()
    }

    /// Returns an optional reference to the agent's task scheduler.
    fn scheduler(&self) -> Option<&TaskScheduler> {
        self.scheduler.as_ref()
    }

    /// Returns the agent's registered capabilities.
    fn capabilities(&self) -> &HashSet<Capability> {
        &self.capabilities
    }

    /// Returns the context manager tracking recent communication and focus.
    fn context(&self) -> &ContextManager {
        &self.context
    }

    /// Returns the list of current tasks or tasks.
    fn tasks(&self) -> &Vec<Task> {
        &self.tasks
    }

    fn memory_mut(&mut self) -> &mut Vec<Communication> {
        &mut self.memory
    }

    fn planner_mut(&mut self) -> Option<&mut Planner> {
        self.planner.as_mut()
    }

    fn context_mut(&mut self) -> &mut ContextManager {
        &mut self.context
    }
}
