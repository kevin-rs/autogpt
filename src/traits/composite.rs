use crate::traits::agent::Agent;
use crate::traits::functions::{AsyncFunctions, Functions};
use async_trait::async_trait;
use std::fmt::Debug;

#[async_trait]
pub trait AgentFunctions: Agent + Functions + AsyncFunctions + Send + Sync + Debug {}

impl<T> AgentFunctions for T where T: Agent + Functions + AsyncFunctions + Send + Sync + Debug {}
