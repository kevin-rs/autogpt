use autogpt::common::utils::{Communication, Status};
use autogpt::traits::agent::Agent;
use std::borrow::Cow;

pub struct MockAgent {
    objective: Cow<'static, str>,
    position: Cow<'static, str>,
    status: Status,
    memory: Vec<Communication>,
}

impl MockAgent {
    pub fn new(objective: Cow<'static, str>, position: Cow<'static, str>) -> Self {
        Self {
            objective,
            position,
            status: Status::InDiscovery,
            memory: Vec::new(),
        }
    }
}

impl Agent for MockAgent {
    fn new(objective: Cow<'static, str>, position: Cow<'static, str>) -> Self {
        Self::new(objective, position)
    }

    fn update(&mut self, status: Status) {
        self.status = status;
    }

    fn objective(&self) -> &Cow<'static, str> {
        &self.objective
    }

    fn position(&self) -> &Cow<'static, str> {
        &self.position
    }

    fn status(&self) -> &Status {
        &self.status
    }

    fn memory(&self) -> &Vec<Communication> {
        &self.memory
    }
}

#[test]
fn test_agent_creation() {
    let objective = Cow::Borrowed("Objective");
    let position = Cow::Borrowed("Position");
    let agent = MockAgent::new(objective.clone(), position.clone());

    assert_eq!(*agent.objective(), *objective);
    assert_eq!(*agent.position(), *position);
    assert_eq!(*agent.status(), Status::InDiscovery);
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
