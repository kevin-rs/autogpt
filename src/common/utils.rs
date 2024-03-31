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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
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
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
pub struct Scope {
    /// Indicates if CRUD operations are required.
    pub crud: bool,
    /// Indicates if user login and logout are required.
    pub auth: bool,
    /// Indicates if external URLs are required.
    pub external: bool,
}

/// Represents a fact tasks.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
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

pub fn extract_json_string(text: &str) -> Option<String> {
    if let Some(start_index) = text.find("{\n  \"crud\"") {
        let mut end_index = start_index + 1;
        let mut open_braces_count = 1;

        for (i, c) in text[start_index + 1..].char_indices() {
            match c {
                '{' => open_braces_count += 1,
                '}' => {
                    open_braces_count -= 1;
                    if open_braces_count == 0 {
                        end_index = start_index + i + 2;
                        break;
                    }
                }
                _ => {}
            }
        }

        return Some(text[start_index..end_index].to_string());
    }

    None
}

pub fn extract_array(text: &str) -> Option<String> {
    // Check if the text starts with '[' and ends with ']'
    if text.starts_with('[') && text.ends_with(']') {
        Some(text.to_string())
    } else if let Some(start_index) = text.find("[\"") {
        let mut end_index = start_index + 1;
        let mut open_brackets_count = 1;

        for (i, c) in text[start_index + 1..].char_indices() {
            match c {
                '[' => open_brackets_count += 1,
                ']' => {
                    open_brackets_count -= 1;
                    if open_brackets_count == 0 {
                        end_index = start_index + i + 2;
                        break;
                    }
                }
                _ => {}
            }
        }

        Some(text[start_index..end_index].to_string())
    } else {
        None
    }
}
