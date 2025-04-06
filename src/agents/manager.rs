#![allow(unused)]

//! # `ManagerGPT` agent.
//!

use crate::agents::agent::AgentGPT;
use crate::agents::architect::ArchitectGPT;
use crate::agents::backend::BackendGPT;
#[cfg(feature = "img")]
use crate::agents::designer::DesignerGPT;
use crate::agents::frontend::FrontendGPT;
use crate::agents::git::GitGPT;
use crate::common::utils::strip_code_blocks;
use crate::common::utils::{Communication, Tasks};
use crate::prompts::manager::{FRAMEWORK_MANAGER_PROMPT, LANGUAGE_MANAGER_PROMPT, MANAGER_PROMPT};
use crate::traits::agent::Agent;
use crate::traits::functions::Functions;
use anyhow::Result;
use colored::*;
use gems::Client;
use std::borrow::Cow;
use std::env::var;
use tracing::info;

/// Enum representing different types of GPT agents.
#[derive(Debug, Clone)]
enum AgentType {
    /// Architect GPT agent.
    Architect(ArchitectGPT),
    /// Backend GPT agent.
    Backend(BackendGPT),
    /// Frontend GPT agent.
    Frontend(FrontendGPT),
    /// Designer GPT agent.
    #[cfg(feature = "img")]
    Designer(DesignerGPT),
    /// Git GPT agent.
    #[cfg(feature = "git")]
    Git(GitGPT),
}

impl AgentType {
    /// Asynchronously executes tasks associated with the agent.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to tasks to be executed.
    /// * `execute` - A boolean indicating whether to execute the tasks.
    /// * `max_tries` - Maximum number of attempts to execute tasks.
    ///
    /// # Returns
    ///
    /// (`Result<()>`): Result indicating success or failure of task execution.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in executing tasks.
    ///
    /// # Business Logic
    ///
    /// - Executes tasks associated with the agent based on its type.
    ///
    async fn execute(&mut self, tasks: &mut Tasks, execute: bool, max_tries: u64) -> Result<()> {
        match self {
            AgentType::Architect(agent) => agent.execute(tasks, execute, max_tries).await,
            AgentType::Backend(agent) => agent.execute(tasks, execute, max_tries).await,
            AgentType::Frontend(agent) => agent.execute(tasks, execute, max_tries).await,
            #[cfg(feature = "img")]
            AgentType::Designer(agent) => agent.execute(tasks, execute, max_tries).await,
            #[cfg(feature = "git")]
            AgentType::Git(agent) => agent.execute(tasks, execute, max_tries).await,
        }
    }

    /// Retrieves the position of the agent.
    ///
    /// # Returns
    ///
    /// (`String`): The position of the agent.
    ///
    /// # Business Logic
    ///
    /// - Retrieves the position of the agent based on its type.
    ///
    fn position(&self) -> String {
        match self {
            AgentType::Architect(agent) => agent.get_agent().position().to_string(),
            AgentType::Backend(agent) => agent.get_agent().position().to_string(),
            AgentType::Frontend(agent) => agent.get_agent().position().to_string(),
            AgentType::Git(agent) => agent.get_agent().position().to_string(),
            _ => "Any".to_string(),
        }
    }
}

/// Struct representing a ManagerGPT, responsible for managing different types of GPT agents.
#[derive(Debug)]
#[allow(unused)]
pub struct ManagerGPT {
    /// Represents the GPT agent associated with the manager.
    agent: AgentGPT,
    /// Represents the tasks to be executed by the manager.
    tasks: Tasks,
    /// Represents the programming language used in the tasks.
    language: &'static str,
    /// Represents a collection of GPT agents managed by the manager.
    agents: Vec<AgentType>,
    /// Represents a client for interacting with external services.
    client: Client,
}

impl ManagerGPT {
    /// Constructor function to create a new instance of ManagerGPT.
    ///
    /// # Arguments
    ///
    /// * `objective` - Objective description for ManagerGPT.
    /// * `position` - Position description for ManagerGPT.
    /// * `request` - Description of the user's request.
    /// * `language` - Programming language used in the tasks.
    ///
    /// # Returns
    ///
    /// (`ManagerGPT`): A new instance of ManagerGPT.
    ///
    /// # Business Logic
    ///
    /// - Initializes the GPT agent with the given objective and position.
    /// - Initializes an empty collection of agents.
    /// - Initializes tasks with the provided description.
    /// - Initializes a Gemini client for interacting with Gemini API.
    ///
    pub fn new(
        objective: &'static str,
        position: &'static str,
        request: &str,
        language: &'static str,
    ) -> Self {
        let agent = AgentGPT::new_borrowed(objective, position);

        let agents: Vec<AgentType> = Vec::new();

        // let request = format!("{}\n\nUser Request: {}", MANAGER_PROMPT, request);

        let tasks: Tasks = Tasks {
            description: request.to_string().into(),
            scope: None,
            urls: None,
            frontend_code: None,
            backend_code: None,
            api_schema: None,
        };

        info!(
            "{}",
            format!("[*] {:?}: 🛠️  Getting ready!", agent.position(),)
                .bright_white()
                .bold()
        );

        let model = var("GEMINI_MODEL")
            .unwrap_or("gemini-2.0-flash".to_string())
            .to_owned();

        let api_key = var("GEMINI_API_KEY").unwrap_or_default().to_owned();
        let client = Client::new(&api_key, &model);

        Self {
            agent,
            tasks,
            language,
            agents,
            client,
        }
    }

    /// Adds an agent to the manager.
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent to be added.
    ///
    /// # Business Logic
    ///
    /// - Adds the specified agent to the collection of agents managed by the manager.
    ///
    fn add_agent(&mut self, agent: AgentType) {
        self.agents.push(agent);
    }

    fn spawn_default_agents(&mut self) {
        self.add_agent(AgentType::Architect(ArchitectGPT::new(
            "Creates innovative website designs and user experiences",
            "ArchitectGPT",
        )));
        #[cfg(feature = "img")]
        self.add_agent(AgentType::Designer(DesignerGPT::new(
            "Creates innovative website designs and user experiences",
            "DesignerGPT",
        )));
        self.add_agent(AgentType::Backend(BackendGPT::new(
            "Expertise lies in writing backend code for web servers and JSON databases",
            "BackendGPT",
            self.language,
        )));
        self.add_agent(AgentType::Frontend(FrontendGPT::new(
            "Expertise lies in writing frontend code for Yew rust framework",
            "FrontendGPT",
            self.language,
        )));
        self.add_agent(AgentType::Git(GitGPT::new(
            "Handles git operations like staging and committing code",
            "GitGPT",
        )));
    }

    /// Spawns default agents if the collection is empty.
    ///
    /// # Business Logic
    ///
    /// - Adds default agents to the collection if it is empty.
    ///
    pub async fn execute_prompt(&self, prompt: String) -> Result<String> {
        let gemini_response: String = match self.client.clone().generate_content(&prompt).await {
            Ok(response) => strip_code_blocks(&response),
            Err(_err) => Default::default(),
        };

        Ok(gemini_response)
    }
    /// Asynchronously executes the tasks described by the user request.
    ///
    /// # Arguments
    ///
    /// * `execute` - A boolean indicating whether to execute the tasks.
    /// * `max_tries` - Maximum number of attempts to execute tasks.
    ///
    /// # Returns
    ///
    /// (`Result<()>`): Result indicating success or failure of task execution.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in executing tasks.
    ///
    /// # Business Logic
    ///
    /// - Executes tasks described by the user request using the collection of agents managed by the manager.
    /// - Logs user request, system decisions, and assistant responses.
    /// - Manages retries and error handling during task execution.
    pub async fn execute(&mut self, execute: bool, max_tries: u64) -> Result<()> {
        self.agent.add_communication(Communication {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(format!(
                "Execute tasks with description: '{}'",
                self.tasks.description.clone()
            )),
        });

        info!(
            "{}",
            format!(
                "[*] {:?}: Executing task: {:?}",
                self.agent.position(),
                self.tasks.description.clone()
            )
            .bright_white()
            .bold()
        );

        let language_request = format!(
            "{}\n\nUser Request: {}",
            LANGUAGE_MANAGER_PROMPT,
            self.tasks.description.clone()
        );

        let framework_request = format!(
            "{}\n\nUser Request: {}",
            FRAMEWORK_MANAGER_PROMPT,
            self.tasks.description.clone()
        );

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(
                "Analyzing user request to determine programming language and framework..."
                    .to_string(),
            ),
        });

        let language = self.execute_prompt(language_request).await?;
        let framework = self.execute_prompt(framework_request).await?;

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(format!(
                "Identified Language: '{}', Framework: '{}'",
                language, framework
            )),
        });

        if self.agents.is_empty() {
            self.spawn_default_agents();
            self.agent.add_communication(Communication {
                role: Cow::Borrowed("system"),
                content: Cow::Borrowed("No agents were available. Spawned default agents."),
            });
        }

        for mut agent in self.agents.clone() {
            let request_prompt = format!(
            "{}\n\n\n\nUser Request: {}\n\nAgent Role: {}\nProgramming Language: {}\nFramework: {}\n",
            MANAGER_PROMPT,
            self.tasks.description.clone(),
            agent.position(),
            language,
            framework
        );

            let refined_task = self.execute_prompt(request_prompt).await?;

            self.agent.add_communication(Communication {
                role: Cow::Borrowed("assistant"),
                content: Cow::Owned(format!(
                    "Refined task for '{}': {}",
                    agent.position(),
                    refined_task
                )),
            });

            self.tasks = Tasks {
                description: refined_task.into(),
                scope: None,
                urls: None,
                frontend_code: None,
                backend_code: None,
                api_schema: None,
            };

            let _agent_res = agent.execute(&mut self.tasks, execute, max_tries).await;
        }

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Borrowed("Task execution completed by all agents."),
        });

        info!(
            "{}",
            format!("[*] {:?}: Completed Tasks:", self.agent.position())
                .bright_white()
                .bold()
        );

        Ok(())
    }
}
