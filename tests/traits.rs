use autogpt::common::utils::{Communication, Status};
use autogpt::prelude::*;
use autogpt::traits::agent::Agent;
use std::borrow::Cow;

#[derive(Debug, Default)]
pub struct MockAgent {
    objective: Cow<'static, str>,
    position: Cow<'static, str>,
    status: Status,
    memory: Vec<Communication>,
    tools: Vec<Tool>,
    knowledge: Knowledge,
    planner: Option<Planner>,
    persona: Persona,
    collaborators: Vec<Arc<Mutex<Box<dyn AgentFunctions>>>>,
    reflection: Option<Reflection>,
    scheduler: Option<TaskScheduler>,
    capabilities: HashSet<Capability>,
    context: ContextManager,
    tasks: Vec<Task>,
}

impl Agent for MockAgent {
    fn new(
        objective: std::borrow::Cow<'static, str>,
        position: std::borrow::Cow<'static, str>,
    ) -> Self {
        let mut agent = MockAgent {
            objective: objective.clone(),
            position: position.clone(),
            ..Default::default()
        };
        agent.objective = objective;
        agent.position = position;
        agent
    }

    fn update(&mut self, status: Status) {
        self.status = status;
    }

    fn objective(&self) -> &std::borrow::Cow<'static, str> {
        &self.objective
    }

    fn position(&self) -> &std::borrow::Cow<'static, str> {
        &self.position
    }

    fn status(&self) -> &Status {
        &self.status
    }

    fn memory(&self) -> &Vec<Communication> {
        &self.memory
    }

    fn tools(&self) -> &Vec<Tool> {
        &self.tools
    }

    fn knowledge(&self) -> &Knowledge {
        &self.knowledge
    }

    fn planner(&self) -> Option<&Planner> {
        self.planner.as_ref()
    }

    fn persona(&self) -> &Persona {
        &self.persona
    }

    fn collaborators(&self) -> &Vec<Arc<Mutex<Box<dyn AgentFunctions>>>> {
        &self.collaborators
    }

    fn reflection(&self) -> Option<&Reflection> {
        self.reflection.as_ref()
    }

    fn scheduler(&self) -> Option<&TaskScheduler> {
        self.scheduler.as_ref()
    }

    fn capabilities(&self) -> &std::collections::HashSet<Capability> {
        &self.capabilities
    }

    fn context(&self) -> &ContextManager {
        &self.context
    }

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

#[test]
fn test_agent_creation() {
    let objective = Cow::Borrowed("Objective");
    let position = Cow::Borrowed("Position");
    let agent = MockAgent::new(objective.clone(), position.clone());

    assert_eq!(*agent.objective(), *objective);
    assert_eq!(*agent.position(), *position);
    assert_eq!(*agent.status(), Status::Idle);
    assert!(agent.memory().is_empty());
}

#[test]
fn test_agent_update() {
    let mut agent = MockAgent::new(Cow::Borrowed("Objective"), Cow::Borrowed("Position"));

    agent.update(Status::Active);
    assert_eq!(*agent.status(), Status::Active);

    agent.update(Status::InUnitTesting);
    assert_eq!(*agent.status(), Status::InUnitTesting);
}
