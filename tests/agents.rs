use autogpt::agents::agent::AgentGPT;
use autogpt::common::utils::{Communication, Status};
use autogpt::traits::agent::Agent;
use std::borrow::Cow;

#[test]
fn test_create_agent() {
    let objective = "Creates innovative website designs and user experiences";
    let position = "Lead UX/UI Designer";

    let agent = AgentGPT::new_borrowed(objective, position);

    assert_eq!(*agent.objective(), *objective);
    assert_eq!(*agent.position(), *position);
    assert_eq!(*agent.status(), Status::Idle);
    assert!(agent.memory().is_empty());
}

#[test]
fn test_update_status() {
    let objective = "Develops cutting-edge web applications with advanced features";
    let position = "Lead Web Developer";

    let mut agent = AgentGPT::new_borrowed(objective, position);

    agent.update(Status::Active);
    assert_eq!(*agent.status(), Status::Active);

    agent.update(Status::InUnitTesting);
    assert_eq!(*agent.status(), Status::InUnitTesting);
}

#[test]
fn test_access_properties() {
    let objective = "Creates innovative website designs and user experiences";
    let position = "Lead UX/UI Designer";

    let agent = AgentGPT::new_borrowed(objective, position);

    assert_eq!(*agent.objective(), objective);
    assert_eq!(*agent.position(), position);
}

#[test]
fn test_memory() {
    let objective = "Develops cutting-edge web applications with advanced features";
    let position = "Lead Web Developer";

    let mut agent = AgentGPT::new_borrowed(objective, position);

    assert!(agent.memory().clone().is_empty());

    let communication = Communication {
        role: Cow::Borrowed("Role"),
        content: Cow::Borrowed("Content"),
    };

    agent.add_communication(communication);

    assert_eq!(agent.memory().len(), 1);
    assert_eq!(agent.memory()[0].role, "Role");
    assert_eq!(agent.memory()[0].content, "Content");
}
