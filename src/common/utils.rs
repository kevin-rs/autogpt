use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;

/// Represents a communication between agents.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Communication {
    /// The role of the communication.
    pub role: Cow<'static, str>,
    /// The content of the communication.
    pub content: Cow<'static, str>,
}

/// Represents the status of an agent.
#[derive(Debug, PartialEq, Default, Clone)]
pub enum Status {
    /// Agent is in the discovery phase.
    #[default]
    InDiscovery,
    /// Agent is actively working.
    Active,
    /// Agent is in the unit testing phase.
    InUnitTesting,
    /// Agent has finished its task.
    Completed,
}

/// Represents a route object.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Route {
    /// Indicates if the route is dynamic.
    pub dynamic: Cow<'static, str>,
    /// The HTTP method of the route.
    pub method: Cow<'static, str>,
    /// The request body of the route.
    pub body: Value,
    /// The response of the route.
    pub response: Value,
    /// The route path.
    pub path: Cow<'static, str>,
}

/// Represents the scope of a project.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct Scope {
    /// Indicates if CRUD operations are required.
    pub crud: bool,
    /// Indicates if user login and logout are required.
    pub auth: bool,
    /// Indicates if external URLs are required.
    pub external: bool,
}

/// Represents a fact tasks.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Tasks {
    /// The description of the project.
    pub description: Cow<'static, str>,
    /// The scope of the project.
    pub scope: Option<Scope>,
    /// External URLs required by the project.
    pub urls: Option<Vec<Cow<'static, str>>>,
    /// Backend code of the project.
    pub backend_code: Option<Cow<'static, str>>,
    /// Schema of API endpoints.
    pub api_schema: Option<Vec<Route>>,
}
