use anyhow::{Context, Result};
use colored::*;
use git2::{IndexAddOption, Repository, Signature};
use std::borrow::Cow;
use std::env::var;
use tokio::fs;
use tracing::{debug, error, info};

use crate::agents::agent::AgentGPT;
use crate::common::utils::{
    Capability, ClientType, Communication, ContextManager, Knowledge, Persona, Planner, Reflection,
    Status, Task, TaskScheduler, Tool,
};
use crate::traits::agent::Agent;
use crate::traits::composite::AgentFunctions;
use crate::traits::functions::{Executor, AsyncFunctions, Functions};
use async_trait::async_trait;
use auto_derive::Auto;
use std::fmt;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "mem")]
use {
    crate::common::memory::load_long_term_memory, crate::common::memory::long_term_memory_context,
    crate::common::memory::save_long_term_memory,
};

#[cfg(feature = "oai")]
use {openai_dive::v1::models::FlagshipModel, openai_dive::v1::resources::chat::*};

#[cfg(feature = "gem")]
use gems::{
    chat::ChatBuilder,
    messages::{Content, Message},
    traits::CTrait,
};

#[cfg(feature = "xai")]
use x_ai::{
    chat_compl::{ChatCompletionsRequestBuilder, Message as XaiMessage},
    traits::ChatCompletionsFetcher,
};

#[cfg(feature = "cld")]
use anthropic_ai_sdk::types::message::{
    ContentBlock, CreateMessageParams, Message as AnthMessage, MessageClient,
    RequiredMessageParams, Role,
};

/// Struct representing GitGPT, a thread-safe Git-aware task executor integrated with a GPT agent.
#[allow(dead_code)]
#[derive(Auto)]
pub struct GitGPT {
    /// Path to the working directory used by the Git repository.
    workspace: Cow<'static, str>,
    /// GPT-based agent handling task status and communication.
    agent: AgentGPT,
    /// Represents an OpenAI or Gemini client for interacting with their API.
    client: ClientType,
    /// A handle to the local Git repository.
    repo: Mutex<Repository>,
    /// Git repository path.
    repo_path: String,
}

/// Implements manual cloning for GitGPT.
///
/// # Behavior
///
/// Reopens the repository from the existing workspace path and clones agent state.
impl Clone for GitGPT {
    fn clone(&self) -> Self {
        let repo = Repository::open(&*self.workspace)
            .expect("Failed to reopen Git repository during clone");

        let repo_path = repo.path().to_string_lossy().to_string();
        Self {
            workspace: self.workspace.clone(),
            agent: self.agent.clone(),
            repo: repo.into(),
            client: self.client.clone(),
            repo_path,
        }
    }
}

/// Debug formatting implementation for GitGPT.
///
/// # Output
///
/// Provides formatted output of workspace, agent, and repository path.
impl fmt::Debug for GitGPT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GitGPT")
            .field("workspace", &self.workspace)
            .field("agent", &self.agent)
            .field("repo", &"Repository { ... }")
            .field("repo_path", &self.repo_path)
            .finish()
    }
}

impl Default for GitGPT {
    fn default() -> Self {
        let temp_path = "/tmp/gitgpt";

        let repo =
            Repository::init(temp_path).expect("Failed to initialize default Git repository");

        GitGPT {
            workspace: Cow::Borrowed(temp_path),
            agent: AgentGPT::default(),
            repo: Mutex::new(repo),
            repo_path: temp_path.to_string(),
            client: ClientType::default(),
        }
    }
}

impl GitGPT {
    /// Constructs a new `GitGPT` instance.
    ///
    /// # Arguments
    ///
    /// * `objective` - The goal or mission for the agent.
    /// * `position` - The role or identity of the agent.
    ///
    /// # Returns
    ///
    /// (`GitGPT`): A new GitGPT instance with initialized workspace, agent, and Git repository.
    ///
    /// # Business Logic
    ///
    /// - Sets up the Git workspace directory.
    /// - Initializes or opens a Git repository.
    /// - Creates a GPT agent with the provided objective and position.
    pub async fn new(objective: &'static str, position: &'static str) -> Self {
        let workspace = var("AUTOGPT_WORKSPACE").unwrap_or_else(|_| "workspace/".to_string());

        if !fs::try_exists(&workspace).await.unwrap_or(false) {
            match fs::create_dir_all(&workspace).await {
                Ok(_) => debug!("Directory '{}' created successfully!", workspace),
                Err(e) => error!("Error creating directory '{}': {}", workspace, e),
            }
        } else {
            debug!("Workspace directory '{}' already exists.", workspace);
        }

        let agent = AgentGPT::new_borrowed(objective, position);

        let repo = if fs::try_exists(format!("{}/.git", &workspace))
            .await
            .unwrap_or(false)
        {
            Repository::open(&workspace).expect("Failed to open existing repository")
        } else {
            Repository::init(&workspace).expect("Failed to initialize git repository")
        };
        let repo_path = repo.path().to_string_lossy().to_string();

        info!(
            "{}",
            format!("[*] {:?}: GitGPT initialized.", agent.position())
        );

        Self {
            workspace: workspace.into(),
            repo: Mutex::new(repo),
            agent,
            repo_path,
            client: ClientType::default(),
        }
    }

    /// Generates a Git author signature from the agent's position.
    ///
    /// # Returns
    ///
    /// (`Signature`): A Git signature representing the current agent.
    ///
    /// # Errors
    ///
    /// Panics if signature creation fails.
    fn author_signature(&self) -> Signature<'_> {
        let name = self.agent.position().to_string();
        let email = format!("{}@kevin-rs.dev", name.to_lowercase().replace(" ", "_"));
        Signature::now(&name, &email).expect("Failed to create signature")
    }

    /// Stages all changes in the working directory.
    ///
    /// # Returns
    ///
    /// (`Result<()>`): Ok if successful, error otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if file indexing or writing fails.
    async fn stage_all(&self) -> Result<()> {
        let repo = self.repo.lock().await;
        let mut index = repo.index().context("Failed to get index")?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        index.write().context("Failed to write index")?;
        Ok(())
    }

    /// Commits staged changes with a given message.
    ///
    /// # Arguments
    ///
    /// * `message` - The commit message to include.
    ///
    /// # Returns
    ///
    /// (`Result<()>`): Ok if commit is successful, error otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if writing the tree or commit fails.
    async fn commit_changes(&self, message: &str) -> Result<()> {
        let repo = self.repo.lock().await;

        let sig = self.author_signature();

        let tree_oid = {
            let mut index = repo.index()?;
            index.write_tree()?
        };

        let tree = repo.find_tree(tree_oid)?;

        let parent_commit = match repo.head().ok().and_then(|h| h.target()) {
            Some(oid) => vec![repo.find_commit(oid)?],
            None => vec![],
        };

        let parents: Vec<&git2::Commit> = parent_commit.iter().collect();

        let commit_oid = repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;

        info!(
            "{}",
            format!(
                "[*] {:?}: Commit created: {}",
                self.agent.position(),
                commit_oid
            )
            .bright_blue()
        );

        Ok(())
    }
}

/// Implementation of the `AsyncFunctions` trait for `GitGPT`.
///
/// Provides access to the agent and defines asynchronous task execution,
/// including staging and committing changes in a Git repository.
#[async_trait]
impl Executor for GitGPT {
    /// Executes a Git commit task asynchronously based on agent status.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to task descriptions and metadata.
    /// * `_execute` - Flag to indicate execution logic (unused).
    /// * `_max_tries` - Maximum retries for execution (unused).
    ///
    /// # Returns
    ///
    /// (`Result<()>`): Ok if task executed successfully, error otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if staging or committing changes fails.
    ///
    /// # Business Logic
    ///
    /// - Logs the task description.
    /// - If agent is idle, stages files and creates a commit.
    /// - Updates agent status to completed after successful commit.
    async fn execute<'a>(
        &'a mut self,
        tasks: &'a mut Task,
        _execute: bool,
        _browse: bool,
        _max_tries: u64,
    ) -> Result<()> {
        info!(
            "{}",
            format!(
                "[*] {:?}: Executing Git commit task.",
                self.agent.position()
            )
            .bright_white()
            .bold()
        );

        for task in tasks.description.clone().split("- ") {
            if !task.trim().is_empty() {
                info!("{} {}", "•".bright_white().bold(), task.trim().cyan());
            }
        }

        match self.agent.status() {
            Status::Idle => {
                debug!("Agent is idle, proceeding to stage and commit files.");

                self.stage_all()
                    .await
                    .context("Staging files with git2 failed")?;

                self.commit_changes(&tasks.description)
                    .await
                    .context("Git commit failed")?;

                self.agent.update(Status::Completed);
            }
            _ => {
                debug!(
                    "[*] {:?}: GitGPT status is not Idle. Skipping commit.",
                    self.agent.position()
                );
            }
        }

        Ok(())
    }
}
