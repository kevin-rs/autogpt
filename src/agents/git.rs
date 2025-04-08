use anyhow::{Context, Result};
use colored::*;
use git2::{IndexAddOption, Repository, Signature};
use std::borrow::Cow;
use std::env::var;
use std::fs;
use std::path::Path;
use tracing::{debug, error, info};

use crate::agents::agent::AgentGPT;
use crate::common::utils::Communication;
use crate::common::utils::{Status, Tasks};
use crate::traits::agent::Agent;
use crate::traits::functions::Functions;
use std::fmt;

/// Struct representing GitGPT, a thread-safe Git-aware task executor integrated with a GPT agent.
#[allow(dead_code)]
pub struct GitGPT {
    /// Path to the working directory used by the Git repository.
    workspace: Cow<'static, str>,
    /// GPT-based agent handling task status and communication.
    agent: AgentGPT,
    /// A handle to the local Git repository.
    repo: Repository,
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

        Self {
            workspace: self.workspace.clone(),
            agent: self.agent.clone(),
            repo,
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
            .field("repo_path", &self.repo.path().to_str())
            .finish()
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
    pub fn new(objective: &'static str, position: &'static str) -> Self {
        let workspace = var("AUTOGPT_WORKSPACE").unwrap_or_else(|_| "workspace/".to_string());

        if !Path::new(&workspace).exists() {
            if let Err(e) = fs::create_dir_all(&workspace) {
                error!("Failed to create workspace '{}': {}", workspace, e);
            }
        }

        let agent = AgentGPT::new_borrowed(objective, position);

        let repo = if Path::new(&format!("{}/.git", &workspace)).exists() {
            Repository::open(&workspace).expect("Failed to open existing repository")
        } else {
            Repository::init(&workspace).expect("Failed to initialize git repository")
        };

        info!(
            "{}",
            format!("[*] {:?}: GitGPT initialized.", agent.position())
                .bright_green()
                .bold()
        );

        Self {
            workspace: workspace.into(),
            repo,
            agent,
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
    fn stage_all(&self) -> Result<()> {
        let mut index = self.repo.index().context("Failed to get index")?;
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
    fn commit_changes(&self, message: &str) -> Result<()> {
        let sig = self.author_signature();

        let tree_oid = {
            let mut index = self.repo.index()?;
            index.write_tree()?
        };

        let tree = self.repo.find_tree(tree_oid)?;

        let parent_commit = match self.repo.head().ok().and_then(|h| h.target()) {
            Some(oid) => vec![self.repo.find_commit(oid)?],
            None => vec![],
        };

        let parents: Vec<&git2::Commit> = parent_commit.iter().collect();

        let commit_oid = self
            .repo
            .commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)?;

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

/// Implementation of the `Functions` trait for `GitGPT`.
///
/// Provides access to the agent and defines asynchronous task execution,
/// including staging and committing changes in a Git repository.
impl Functions for GitGPT {
    /// Returns a reference to the internal agent.
    ///
    /// # Returns
    ///
    /// (`&AgentGPT`): Reference to the embedded GPT agent.
    fn get_agent(&self) -> &AgentGPT {
        &self.agent
    }

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
    async fn execute(
        &mut self,
        tasks: &mut Tasks,
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

                self.stage_all().context("Staging files with git2 failed")?;

                self.commit_changes(&tasks.description)
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
    /// Saves a communication to long-term memory for the agent.
    ///
    /// # Arguments
    ///
    /// * `communication` - The communication to save, which contains the role and content.
    ///
    /// # Returns
    ///
    /// (`Result<()>`): Result indicating the success or failure of saving the communication.
    ///
    /// # Business Logic
    ///
    /// - This method uses the `save_long_term_memory` util function to save the communication into the agent's long-term memory.
    /// - The communication is embedded and stored using the agent's unique ID as the namespace.
    /// - It handles the embedding and metadata for the communication, ensuring it's stored correctly.
    async fn save_ltm(&mut self, _communication: Communication) -> Result<()> {
        // dummy impl cz of no ai client
        Ok(())
    }

    /// Retrieves all communications stored in the agent's long-term memory.
    ///
    /// # Returns
    ///
    /// (`Result<Vec<Communication>>`): A result containing a vector of communications retrieved from the agent's long-term memory.
    ///
    /// # Business Logic
    ///
    /// - This method fetches the stored communications for the agent by interacting with the `load_long_term_memory` function.
    /// - The function will return a list of communications that are indexed by the agent's unique ID.
    /// - It handles the retrieval of the stored metadata and content for each communication.
    async fn get_ltm(&self) -> Result<Vec<Communication>> {
        // dummy impl cz of no ai client
        Ok(vec![
            Communication {
                role: Cow::Borrowed("system"),
                content: Cow::Borrowed("System initialized."),
            },
            Communication {
                role: Cow::Borrowed("user"),
                content: Cow::Borrowed("Hello, autogpt!"),
            },
        ])
    }

    /// Retrieves the concatenated context of all communications in the agent's long-term memory.
    ///
    /// # Returns
    ///
    /// (`String`): A string containing the concatenated role and content of all communications stored in the agent's long-term memory.
    ///
    /// # Business Logic
    ///
    /// - This method calls the `long_term_memory_context` function to generate a string representation of the agent's entire long-term memory.
    /// - The context string is composed of each communication's role and content, joined by new lines.
    /// - It provides a quick overview of the agent's memory in a human-readable format.
    async fn ltm_context(&self) -> String {
        // dummy impl cz of no ai client
        let comms = [
            Communication {
                role: Cow::Borrowed("system"),
                content: Cow::Borrowed("System initialized."),
            },
            Communication {
                role: Cow::Borrowed("user"),
                content: Cow::Borrowed("Hello, autogpt!"),
            },
        ];

        comms
            .iter()
            .map(|c| format!("{}: {}", c.role, c.content))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
