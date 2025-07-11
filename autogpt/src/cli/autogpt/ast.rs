use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub ai_provider: String,
    pub model: String,
    pub position: String,
    pub role: String,
    pub prompt: String,
}
