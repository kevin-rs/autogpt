use kevin::agents::agent::AgentKevin;
use kevin::common::utils::{Communication, Status};
use kevin::traits::agent::Agent;
use std::borrow::Cow;

#[test]
fn test_create_agent() {
    let objective = "Objective";
    let position = "Position";
    let agent = AgentKevin::new_borrowed(objective, position);

    assert_eq!(*agent.objective(), *objective);
    assert_eq!(*agent.position(), *position);
    assert_eq!(*agent.status(), Status::InDiscovery);
    assert!(agent.memory().is_empty());
}

#[test]
fn test_update_status() {
    let mut agent = AgentKevin::new_borrowed("Objective", "Position");

    agent.update(Status::Active);
    assert_eq!(*agent.status(), Status::Active);

    agent.update(Status::InUnitTesting);
    assert_eq!(*agent.status(), Status::InUnitTesting);
}

#[test]
fn test_access_properties() {
    let agent = AgentKevin::new_borrowed("Objective", "Position");

    assert_eq!(*agent.objective(), "Objective");
    assert_eq!(*agent.position(), "Position");
}

#[test]
fn test_memory() {
    let mut agent = AgentKevin::new_borrowed("Objective", "Position");

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
