use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Network {
    async fn heartbeat(&self);
    async fn broadcast(&self, payload: &str) -> Result<()>;
}
