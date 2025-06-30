//! # `BackendGPT` agent.
//!
//! This module provides functionality for generating backend code for web servers
//! and JSON databases based on prompts using Gemini or OpenAI API. The `BackendGPT` agent
//! understands user requirements and produces code snippets in various programming
//! languages commonly used for backend development.
//!
//! # Example - Generating backend code:
//!
//! ```rust
//! use autogpt::agents::backend::BackendGPT;
//! use autogpt::common::utils::Task;
//! use autogpt::traits::functions::Functions;
//! use autogpt::traits::functions::AsyncFunctions;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut backend_agent = BackendGPT::new(
//!         "Generate backend code",
//!         "Backend Developer",
//!         "rust",
//!     ).await;
//!
//!     let mut tasks = Task {
//!         description: "Create REST API endpoints for user authentication".into(),
//!         scope: None,
//!         urls: None,
//!         frontend_code: None,
//!         backend_code: None,
//!         api_schema: None,
//!     };
//!
//!     if let Err(err) = backend_agent.execute(&mut tasks, true, false, 3).await {
//!         eprintln!("Error executing backend tasks: {:?}", err);
//!     }
//! }
//! ```
//!
#![allow(unreachable_code)]

use crate::agents::agent::AgentGPT;
#[cfg(feature = "cli")]
use crate::common::utils::spinner;
#[allow(unused_imports)]
use crate::common::utils::{
    Capability, ClientType, Communication, ContextManager, GenerationOutput, Goal, Knowledge,
    OutputKind, Persona, Planner, Reflection, Route, Scope, Status, Task, TaskScheduler, Tool,
    extract_array, strip_code_blocks,
};
use crate::prompts::backend::{
    API_ENDPOINTS_PROMPT, FIX_CODE_PROMPT, IMPROVED_WEBSERVER_CODE_PROMPT, WEBSERVER_CODE_PROMPT,
};
use crate::traits::agent::Agent;
use crate::traits::composite::AgentFunctions;
use crate::traits::functions::{Executor, AsyncFunctions, Functions};
use auto_derive::Auto;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::sync::Mutex;
// use std::thread::sleep;

use anyhow::Result;
use async_trait::async_trait;
use colored::*;
use reqwest::Client as ReqClient;
use std::borrow::Cow;
use std::env::var;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tokio::process::Child;
use tokio::process::Command;
use tracing::{debug, error, info, warn};
use webbrowser::{Browser, BrowserOptions, open_browser_with_options};

#[cfg(feature = "mem")]
use {
    crate::common::memory::load_long_term_memory, crate::common::memory::long_term_memory_context,
    crate::common::memory::save_long_term_memory,
};

#[cfg(feature = "oai")]
use {openai_dive::v1::models::FlagshipModel, openai_dive::v1::resources::chat::*};

#[cfg(feature = "cld")]
use anthropic_ai_sdk::types::message::{
    ContentBlock, CreateMessageParams, Message as AnthMessage, MessageClient,
    RequiredMessageParams, Role,
};

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

/// Struct representing a BackendGPT, which manages backend development tasks using GPT.
#[derive(Debug, Clone, Default, Auto)]
#[allow(dead_code)]
pub struct BackendGPT {
    /// Represents the workspace directory path for BackendGPT.
    workspace: Cow<'static, str>,
    /// Represents the GPT agent responsible for handling backend tasks.
    agent: AgentGPT,
    /// Represents an OpenAI or Gemini client for interacting with their API.
    client: ClientType,
    /// Represents a client for making HTTP requests.
    req_client: ReqClient,
    /// Represents the bugs found in the codebase, if any.
    bugs: Option<Cow<'static, str>>,
    /// Represents the programming language used for backend development.
    language: &'static str,
    /// Represents the number of bugs found in the codebase.
    nb_bugs: u64,
}

impl BackendGPT {
    /// Constructor function to create a new instance of `BackendGPT`.
    ///
    /// # Arguments
    ///
    /// * `objective` - Objective description for `BackendGPT`.
    /// * `position` - Position description for `BackendGPT`.
    /// * `language` - Programming language used for backend development.
    ///
    /// # Returns
    ///
    /// (`BackendGPT`): A new instance of `BackendGPT`.
    ///
    /// # Business Logic
    ///
    /// - Constructs the workspace directory path for `BackendGPT`.
    /// - Initializes backend projects based on the specified language.
    /// - Initializes the GPT agent with the given objective and position.
    /// - Creates clients for interacting with Gemini or OpenAI API and making HTTP requests.
    #[allow(unused)]
    pub async fn new(
        objective: &'static str,
        position: &'static str,
        language: &'static str,
    ) -> Self {
        let base_workspace = var("AUTOGPT_WORKSPACE").unwrap_or_else(|_| "workspace".to_string());
        let workspace = format!("{}/backend", base_workspace);

        if !fs::try_exists(&workspace).await.unwrap_or(false) {
            match fs::create_dir_all(&workspace).await {
                Ok(_) => debug!("Directory '{}' created successfully!", workspace),
                Err(e) => error!("Error creating directory '{}': {}", workspace, e),
            }
        } else {
            debug!("Workspace directory '{}' already exists.", workspace);
        }

        info!(
            "{}",
            format!("[*] {:?}: 🛠️  Getting ready!", position)
                .bright_white()
                .bold()
        );

        match language {
            "rust" => {
                if !Path::new(&format!("{}/Cargo.toml", workspace)).exists() {
                    let cargo_new = Command::new("cargo").arg("init").arg(&workspace).spawn();

                    match cargo_new {
                        Ok(_) => debug!("Cargo project initialized successfully."),
                        Err(e) => error!("Error initializing Cargo project: {}", e),
                    }
                }

                let template_path = format!("{}/src/template.rs", workspace);
                if !Path::new(&template_path).exists() {
                    if let Err(e) = fs::write(&template_path, "").await {
                        error!("Error creating file '{}': {}", template_path, e);
                    } else {
                        debug!("File '{}' created successfully.", template_path);
                    }
                }
            }

            "python" => {
                let files = ["main.py", "template.py"];
                for file in files.iter() {
                    let full_path = format!("{}/{}", workspace, file);
                    if !Path::new(&full_path).exists() {
                        if let Err(e) = fs::write(&full_path, "").await {
                            error!("Error creating file '{}': {}", full_path, e);
                        } else {
                            debug!("File '{}' created successfully.", full_path);
                        }
                    }
                }
            }

            "javascript" => {
                if !Path::new(&format!("{}/package.json", workspace)).exists() {
                    let npx_install = Command::new("npx")
                        .arg("create-react-app")
                        .arg(&workspace)
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .spawn();

                    match npx_install {
                        Ok(mut child) => match child.wait().await {
                            Ok(status) => {
                                if status.success() {
                                    debug!("React JS project initialized successfully.");
                                } else {
                                    error!("Failed to initialize React JS project.");
                                }
                            }
                            Err(e) => {
                                error!("Error waiting for process: {}", e);
                            }
                        },
                        Err(e) => {
                            error!("Error initializing React JS project: {}", e);
                        }
                    }
                }

                let template_path = format!("{}/src/template.js", workspace);
                if !Path::new(&template_path).exists() {
                    if let Err(e) = fs::write(&template_path, "").await {
                        error!("Error creating file '{}': {}", template_path, e);
                    } else {
                        debug!("File '{}' created successfully.", template_path);
                    }
                }
            }

            _ => panic!(
                "Unsupported language '{}'. Consider opening an issue/PR.",
                language
            ),
        }

        let agent: AgentGPT = AgentGPT::new_borrowed(objective, position);

        let client = ClientType::from_env();

        let req_client: ReqClient = ReqClient::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap();

        Self {
            workspace: workspace.into(),
            agent,
            client,
            req_client,
            bugs: None,
            language,
            nb_bugs: 0,
        }
    }
    pub async fn generate(
        &mut self,
        prompt: &str,
        tasks: &mut Task,
        output_type: OutputKind,
    ) -> Result<GenerationOutput> {
        #[cfg(feature = "mem")]
        {
            self.agent.memory = self.get_ltm().await?;
        }

        let request: String = format!(
            "{}\n\nTask Description: {}\nPrevious Conversation: {:?}",
            prompt,
            tasks.description,
            self.agent.memory(),
        );

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(request.clone()),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("user"),
                    content: Cow::Owned(request.clone()),
                })
                .await;
        }

        #[allow(unused)]
        let mut response_text = String::new();

        #[cfg(any(feature = "oai", feature = "gem", feature = "cld"))]
        {
            response_text = self.send_request(&request).await?;
        }
        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(response_text.clone()),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(response_text.clone()),
                })
                .await;
        }

        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        match output_type {
            OutputKind::Text => Ok(GenerationOutput::Text(strip_code_blocks(&response_text))),
            OutputKind::UrlList => {
                let urls: Vec<Cow<'static, str>> =
                    serde_json::from_str(&extract_array(&response_text).unwrap_or_default())?;
                tasks.urls = Some(urls.clone());
                self.agent.update(Status::InUnitTesting);
                Ok(GenerationOutput::UrlList(urls))
            }
            OutputKind::Scope => {
                let scope: Scope = serde_json::from_str(&strip_code_blocks(&response_text))?;
                Ok(GenerationOutput::Scope(scope))
            }
        }
    }

    /// Asynchronously generates backend code based on tasks and logs the interaction.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to tasks to be processed.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): Result containing the generated backend code.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in reading the template file,
    /// generating content via the Gemini or OpenAI API, or writing the output file.
    ///
    /// # Business Logic
    ///
    /// - Determines the file path based on the specified language.
    /// - Reads the template code from the specified file.
    /// - Constructs a request using the template code and project description.
    /// - Sends the request to the Gemini or OpenAI API to generate backend code.
    /// - Logs the user request and assistant response as communication history in the agent's memory.
    /// - Writes the generated backend code to the appropriate file based on language.
    /// - Updates the task's backend code and the agent's status to `Completed`.
    pub async fn generate_backend_code(&mut self, tasks: &mut Task) -> Result<String> {
        let path = self.workspace.clone();

        let backend_path = match self.language {
            "rust" => format!("{}/{}", path, "src/main.rs"),
            "python" => format!("{}/{}", path, "main.py"),
            "javascript" => format!("{}/{}", path, "src/index.js"),
            _ => panic!("Unsupported language, consider opening an Issue/PR"),
        };

        let template = fs::read_to_string(&backend_path).await?;

        let prompt = format!(
            "{}\n\nCode Template: {}\nProject Description: {}",
            WEBSERVER_CODE_PROMPT, template, tasks.description
        );

        let output = self.generate(&prompt, tasks, OutputKind::Text).await?;

        let code = match output {
            GenerationOutput::Text(code) => code,
            _ => {
                return Err(anyhow::anyhow!(
                    "Expected text output for backend code generation"
                ));
            }
        };

        fs::write(&backend_path, &code).await?;
        tasks.backend_code = Some(code.clone().into());

        self.agent.update(Status::Completed);
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(code)
    }

    /// Asynchronously improves existing backend code based on tasks,
    /// while logging communication between the agent and the AI.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to tasks to be processed.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): Result containing the improved backend code.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in improving the backend code.
    ///
    /// # Business Logic
    ///
    /// - Constructs a request based on the existing backend code and project description.
    /// - Logs the user's request as a `Communication`.
    /// - Sends the request to the Gemini or OpenAI API to generate improved code.
    /// - Logs the AI's response as a `Communication`.
    /// - Writes the improved backend code to the appropriate file.
    /// - Updates tasks and agent status accordingly.
    pub async fn improve_backend_code(&mut self, tasks: &mut Task) -> Result<String> {
        #[cfg(feature = "mem")]
        {
            self.agent.memory = self.get_ltm().await?;
        }

        let code_template = tasks.backend_code.clone().unwrap_or_default();
        let request = format!(
            "{}\n\nCode Template: {}\nProject Description: {}",
            IMPROVED_WEBSERVER_CODE_PROMPT, code_template, tasks.description
        );

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(request.clone()),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("user"),
                    content: Cow::Owned(request.clone()),
                })
                .await;
        }

        #[allow(unused)]
        let mut response_text = String::new();

        #[cfg(any(feature = "oai", feature = "gem", feature = "cld"))]
        {
            response_text = self.send_request(&request).await?;
        }

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(response_text.clone()),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(response_text.clone()),
                })
                .await;
        }

        let cleaned_code = strip_code_blocks(&response_text);

        let backend_path = match self.language {
            "rust" => format!("{}/src/main.rs", self.workspace),
            "python" => format!("{}/main.py", self.workspace),
            "javascript" => format!("{}/src/index.js", self.workspace),
            _ => return Err(anyhow::anyhow!("Unsupported language")),
        };

        debug!(
            "[*] {:?}: Writing to {}",
            self.agent.position(),
            backend_path
        );

        fs::write(&backend_path, &cleaned_code).await?;

        tasks.backend_code = Some(cleaned_code.clone().into());

        self.agent.update(Status::Completed);

        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(cleaned_code)
    }

    /// Asynchronously fixes bugs in the backend code based on tasks,
    /// while logging communication between the agent and the AI.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to tasks to be processed.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): Result containing the fixed backend code.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in fixing the backend code bugs.
    ///
    /// # Business Logic
    ///
    /// - Constructs a request based on the buggy backend code and project description.
    /// - Logs the request as a user `Communication`.
    /// - Sends the request to the Gemini or OpenAI API to generate content for fixing bugs.
    /// - Logs the response or any errors as assistant `Communication`s.
    /// - Writes the fixed backend code to the appropriate file.
    /// - Updates tasks and agent status accordingly.
    pub async fn fix_code_bugs(&mut self, tasks: &mut Task) -> Result<String> {
        #[cfg(feature = "mem")]
        {
            self.agent.memory = self.get_ltm().await?;
        }

        let buggy_code = tasks.backend_code.clone().unwrap_or_default();
        let bugs = self.bugs.clone().unwrap_or_default();
        let request = format!(
            "{}\n\nBuggy Code: {}\nBugs: {}\n\nFix all bugs.",
            FIX_CODE_PROMPT, buggy_code, bugs
        );

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(request.clone()),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("user"),
                    content: Cow::Owned(request.clone()),
                })
                .await;
        }

        #[allow(unused)]
        let mut response_text = String::new();

        #[cfg(any(feature = "oai", feature = "gem", feature = "cld"))]
        {
            response_text = self.send_request(&request).await?;
        }

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(response_text.clone()),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(response_text.clone()),
                })
                .await;
        }

        let cleaned_code = strip_code_blocks(&response_text);

        let workspace = &self.workspace;
        let backend_path = match self.language {
            "rust" => format!("{}/src/main.rs", workspace),
            "python" => format!("{}/main.py", workspace),
            "javascript" => format!("{}/src/index.js", workspace),
            _ => return Err(anyhow::anyhow!("Unsupported language")),
        };

        debug!(
            "[*] {:?}: Writing to {}",
            self.agent.position(),
            backend_path
        );

        fs::write(&backend_path, &cleaned_code).await?;

        tasks.backend_code = Some(cleaned_code.clone().into());

        self.agent.update(Status::Completed);
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(cleaned_code)
    }

    /// Asynchronously retrieves routes JSON from the backend code,
    /// while logging communication between the agent and the AI.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): Result containing the routes JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in retrieving routes JSON.
    ///
    /// # Business Logic
    ///
    /// - Reads the backend code from the appropriate file.
    /// - Constructs a request with the backend code.
    /// - Logs the user's request as a `Communication`.
    /// - Sends the request to the Gemini or OpenAI API to generate content for routes JSON.
    /// - Logs the AI's response as a `Communication`.
    /// - Updates agent status accordingly.
    pub async fn get_routes_json(&mut self) -> Result<String> {
        #[cfg(feature = "mem")]
        {
            self.agent.memory = self.get_ltm().await?;
        }

        let path = self.workspace.clone();
        let full_path = match self.language {
            "rust" => format!("{}/src/main.rs", path),
            "python" => format!("{}/main.py", path),
            "javascript" => format!("{}/src/index.js", path),
            _ => return Err(anyhow::anyhow!("Unsupported language")),
        };

        debug!(
            "[*] {:?}: Reading from {}",
            self.agent.position(),
            full_path
        );

        let backend_code = fs::read_to_string(full_path).await?;
        let request = format!(
            "{}\n\nHere is the backend code with all routes:{}",
            API_ENDPOINTS_PROMPT, backend_code
        );

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(request.clone()),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("user"),
                    content: Cow::Owned(request.clone()),
                })
                .await;
        }
        #[allow(unused)]
        let mut response_text = String::new();

        #[cfg(any(feature = "oai", feature = "gem", feature = "cld"))]
        {
            response_text = self.send_request(&request).await?;
        }

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(response_text.clone()),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(response_text.clone()),
                })
                .await;
        }

        self.agent.update(Status::Completed);
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(strip_code_blocks(&response_text))
    }

    pub fn think(&self) -> String {
        let objective = self.agent.objective();
        format!("How to build and test backend for '{}'", objective)
    }

    pub fn plan(&mut self, _context: String) -> Goal {
        let mut goals = vec![
            Goal {
                description: "Generate backend code".into(),
                priority: 1,
                completed: false,
            },
            Goal {
                description: "Fix code bugs if any".into(),
                priority: 2,
                completed: false,
            },
            Goal {
                description: "Run unit tests and backend server".into(),
                priority: 3,
                completed: false,
            },
        ];

        goals.sort_by_key(|g| g.priority);

        if let Some(planner) = self.agent.planner_mut() {
            if planner.current_plan.is_empty() {
                for g in goals.into_iter().rev() {
                    planner.current_plan.push(g);
                }
            }

            if let Some(next_goal) = planner.current_plan.iter().rev().find(|g| !g.completed) {
                return next_goal.clone();
            }
        }

        Goal {
            description: "Default backend task".into(),
            priority: 1,
            completed: false,
        }
    }

    pub async fn act(
        &mut self,
        goal: Goal,
        tasks: &mut Task,
        execute: bool,
        max_tries: u64,
    ) -> Result<()> {
        info!(
            "{}",
            format!(
                "[*] {:?}: Executing goal: {}",
                self.agent.position(),
                goal.description
            )
            .cyan()
            .bold()
        );

        match goal.description.as_str() {
            "Generate backend code" => {
                self.generate_or_improve_code(tasks).await?;
                self.agent.update(Status::Active);
            }
            "Fix code bugs if any" => {
                if self.nb_bugs > 0 {
                    self.fix_code_bugs(tasks).await?;
                } else {
                    self.improve_backend_code(tasks).await?;
                }
                self.agent.update(Status::InUnitTesting);
            }
            "Run unit tests and backend server" => {
                self.unit_test_and_run_backend(tasks, execute, max_tries)
                    .await?;
                self.agent.update(Status::Completed);
            }
            _ => {
                warn!(
                    "{}",
                    format!(
                        "[*] {:?}: Unknown goal: {}",
                        self.agent.position(),
                        goal.description
                    )
                    .yellow()
                );
            }
        }

        Ok(())
    }

    pub fn reflect(&mut self) {
        let entry = format!(
            "Reflection on backend task for '{}'",
            self.agent.objective()
        );

        self.agent.memory_mut().push(Communication {
            role: Cow::Borrowed("assistant"),
            content: entry.clone().into(),
        });

        self.agent
            .context_mut()
            .recent_messages
            .push(Communication {
                role: Cow::Borrowed("assistant"),
                content: entry.into(),
            });

        if let Some(reflection) = self.agent.reflection() {
            let feedback = (reflection.evaluation_fn)(&self.agent);
            info!(
                "{}",
                format!(
                    "[*] {:?}: Self Reflection: {}",
                    self.agent.position(),
                    feedback
                )
                .blue()
            );
        }
    }

    pub fn has_completed_objective(&self) -> bool {
        if let Some(planner) = self.planner() {
            planner.current_plan.iter().all(|g| g.completed)
        } else {
            false
        }
    }

    pub fn mark_goal_complete(&mut self, goal: Goal) {
        if let Some(planner) = self.planner_mut() {
            for g in &mut planner.current_plan {
                if g.description == goal.description {
                    g.completed = true;
                }
            }
        }
    }

    fn display_task_info(&self, tasks: &Task) {
        for task in tasks.clone().description.clone().split("- ") {
            if !task.trim().is_empty() {
                info!("{} {}", "•".bright_white().bold(), task.trim().cyan());
            }
        }
    }

    async fn open_docs_in_browser(&self) {
        let _ = open_browser_with_options(
            Browser::Default,
            "http://127.0.0.1:8000/docs",
            BrowserOptions::new().with_suppress_output(false),
        );
    }

    async fn generate_or_improve_code(&mut self, tasks: &mut Task) -> Result<()> {
        if self.nb_bugs == 0 {
            self.generate_backend_code(tasks).await?;
        } else {
            self.improve_backend_code(tasks).await?;
        }
        Ok(())
    }

    async fn unit_test_and_run_backend(
        &mut self,
        tasks: &mut Task,
        execute: bool,
        max_tries: u64,
    ) -> Result<()> {
        info!(
            "{}",
            format!(
                "[*] {:?}: Backend Code Unit Testing...",
                self.agent.position()
            )
            .bright_white()
            .bold()
        );

        if !execute {
            warn!(
                "{}",
                format!(
                    "[*] {:?}: Code not safe to proceed, skipping execution...",
                    self.agent.position()
                )
                .bright_yellow()
                .bold()
            );
            return Ok(());
        }

        let path = &self.workspace.to_string();

        let result = self.build_and_run_backend(path).await?;

        if let Some(mut child) = result {
            let mut stderr_output = String::new();
            if let Some(mut stderr) = child.stderr.take() {
                stderr.read_to_string(&mut stderr_output).await?;
            }

            if !stderr_output.trim().is_empty() {
                self.nb_bugs += 1;
                self.bugs = Some(stderr_output.into());

                if self.nb_bugs > max_tries {
                    error!(
                        "{}",
                        format!(
                            "[*] {:?}: Too many bugs detected. Please debug manually.",
                            self.agent.position()
                        )
                        .bright_red()
                        .bold()
                    );
                    return Ok(());
                }

                self.agent.update(Status::Active);
                return Ok(());
            } else {
                self.nb_bugs = 0;
                info!(
                    "{}",
                    format!(
                        "[*] {:?}: Backend server build successful...",
                        self.agent.position()
                    )
                    .bright_white()
                    .bold()
                );
            }

            let endpoints = self.get_routes_json().await?;

            let api_endpoints: Vec<Route> =
                serde_json::from_str(&endpoints).expect("Failed to decode API Endpoints");

            let filtered_endpoints: Vec<Route> = api_endpoints
                .iter()
                .filter(|&route| route.method == "get" && route.dynamic == "false")
                .cloned()
                .collect();

            tasks.api_schema = Some(filtered_endpoints.clone());

            info!(
                "{}",
                format!(
                    "[*] {:?}: Starting web server to test endpoints...",
                    self.agent.position()
                )
                .bright_white()
                .bold()
            );

            for endpoint in filtered_endpoints {
                info!(
                    "{}",
                    format!(
                        "[*] {:?}: Testing endpoint: {}",
                        self.agent.position(),
                        endpoint.path
                    )
                    .bright_white()
                    .bold()
                );

                let url = format!("http://127.0.0.1:8080{}", endpoint.path);
                let status_code = self.req_client.get(url).send().await?.status();

                if status_code != 200 {
                    info!(
                        "{}",
                        format!(
                            "[*] {:?}: Endpoint failed: {}. Needs further investigation.",
                            self.agent.position(),
                            endpoint.path
                        )
                        .bright_white()
                        .bold()
                    );
                }
            }

            let _ = child.kill().await;

            let backend_path = format!("{}/api.json", path);
            fs::write(&backend_path, endpoints).await?;

            info!(
                "{}",
                format!(
                    "[*] {:?}: Backend testing complete. Results saved to api.json",
                    self.agent.position()
                )
                .bright_white()
                .bold()
            );
        } else {
            error!(
                "{}",
                format!(
                    "[*] {:?}: Failed to build or run backend project.",
                    self.agent.position()
                )
                .bright_red()
                .bold()
            );
        }

        Ok(())
    }

    async fn build_and_run_backend(&self, path: &str) -> Result<Option<Child>> {
        match self.language {
            "rust" => self.build_and_run_rust_backend(path).await,
            "python" => self.build_and_run_python_backend(path).await,
            "javascript" => self.build_and_run_js_backend(path).await,
            _ => Ok(None),
        }
    }

    async fn build_and_run_rust_backend(&self, path: &str) -> Result<Option<Child>> {
        let build_output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--verbose")
            .current_dir(path)
            .output()
            .await
            .expect("Failed to build backend");

        if build_output.status.success() {
            let child = Command::new("timeout")
                .arg("10s")
                .arg("cargo")
                .arg("run")
                .arg("--release")
                .arg("--verbose")
                .current_dir(path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to run backend");
            Ok(Some(child))
        } else {
            Ok(None)
        }
    }

    async fn build_and_run_python_backend(&self, path: &str) -> Result<Option<Child>> {
        let venv_path = format!("{}/.venv", path);
        let pip_path = format!("{}/bin/pip", venv_path);
        let venv_exists = Path::new(&venv_path).exists();

        if !venv_exists {
            let create_venv = Command::new("python3")
                .arg("-m")
                .arg("venv")
                .arg(&venv_path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();

            if let Ok(status) = create_venv.await {
                if status.success() {
                    let main_py_path = format!("{}/main.py", path);
                    let main_py_content = fs::read_to_string(&main_py_path)
                        .await
                        .expect("Failed to read main.py");

                    let mut packages = vec![];

                    for line in main_py_content.lines() {
                        if line.starts_with("from ") || line.starts_with("import ") {
                            let parts: Vec<&str> = line.split_whitespace().collect();

                            if let Some(pkg) = parts.get(1) {
                                let root_pkg = pkg.split('.').next().unwrap_or(pkg);
                                if !packages.contains(&root_pkg) {
                                    packages.push(root_pkg);
                                }
                            }
                        }
                    }
                    if !packages.is_empty() {
                        if !packages.contains(&"uvicorn") {
                            packages.push("uvicorn");
                        }
                        if !packages.contains(&"httpx") {
                            packages.push("httpx");
                        }
                        for pkg in &packages {
                            let install_status = Command::new(&pip_path)
                                .arg("install")
                                .arg(pkg)
                                .stdout(Stdio::null())
                                .stderr(Stdio::null())
                                .status();

                            match install_status.await {
                                Ok(status) if status.success() => {
                                    info!(
                                        "{}",
                                        format!(
                                            "[*] {:?}: Successfully installed Python package '{}'",
                                            self.agent.position(),
                                            pkg
                                        )
                                        .bright_white()
                                        .bold()
                                    );
                                }
                                Err(e) => {
                                    error!(
                                        "{}",
                                        format!(
                                            "[*] {:?}: Failed to install Python package '{}': {}",
                                            self.agent.position(),
                                            pkg,
                                            e
                                        )
                                        .bright_red()
                                        .bold()
                                    );
                                }
                                _ => {
                                    error!(
                                        "{}",
                                        format!(
                                            "[*] {:?}: Installation of package '{}' exited with an error",
                                            self.agent.position(),
                                            pkg
                                        )
                                        .bright_red()
                                        .bold()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        let run_output = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "timeout {} '.venv/bin/python' -m uvicorn main:app --host 0.0.0.0 --port 8000",
                10
            ))
            .current_dir(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to run the backend application");

        Ok(Some(run_output))
    }

    async fn build_and_run_js_backend(&self, path: &str) -> Result<Option<Child>> {
        let child = Command::new("timeout")
            .arg("10s")
            .arg("node")
            .arg("app.js")
            .current_dir(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to run js backend");
        Ok(Some(child))
    }
    /// Updates the bugs found in the codebase.
    ///
    /// # Arguments
    ///
    /// * `bugs` - Optional description of bugs found in the codebase.
    ///
    /// # Business Logic
    ///
    /// - Updates the bugs field with the provided description.
    ///
    pub fn update_bugs(&mut self, bugs: Option<Cow<'static, str>>) {
        self.bugs = bugs;
    }
}

/// Implementation of the trait `Executor` for `BackendGPT`.
/// Contains additional methods related to backend tasks.
///
/// This trait provides methods for:
///
/// - Retrieving the agent associated with `BackendGPT`.
/// - Executing tasks asynchronously.
///
/// # Business Logic
///
/// - Provides access to the agent associated with the `BackendGPT` instance.
/// - Executes tasks asynchronously based on the current status of the agent.
/// - Handles task execution including code generation, bug fixing, and testing.
/// - Manages retries and error handling during task execution.
#[async_trait]
impl Executor for BackendGPT {
    /// Asynchronously executes tasks associated with BackendGPT.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to tasks to be executed.
    /// * `execute` - A boolean indicating whether to execute the tasks.
    /// * `browse` - Whether to open the API docs in a browser.
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
    /// - Executes tasks asynchronously based on the current status of the agent.
    /// - Handles task execution including code generation, bug fixing, and testing.
    /// - Manages retries and error handling during task execution.
    ///
    async fn execute<'a>(
        &'a mut self,
        tasks: &'a mut Task,
        execute: bool,
        browse: bool,
        max_tries: u64,
    ) -> Result<()> {
        self.agent.update(Status::Idle);
        info!(
            "{}",
            format!("[*] {:?}: Executing task:", self.agent.position())
                .bright_white()
                .bold()
        );

        self.display_task_info(tasks);

        if browse {
            #[cfg(feature = "cli")]
            let pb = spinner("Opening documentation in browser...");
            self.open_docs_in_browser().await;
            #[cfg(feature = "cli")]
            pb.finish_with_message("Documentation opened.");
        }

        while self.agent.status() != &Status::Completed {
            #[cfg(feature = "cli")]
            let pb = spinner("Thinking...");
            let context = self.think();
            #[cfg(feature = "cli")]
            pb.finish_with_message("Thinking complete!");

            #[cfg(feature = "cli")]
            let pb = spinner("Planning...");
            let goal = self.plan(context);
            #[cfg(feature = "cli")]
            pb.finish_with_message("Planning complete!");

            #[cfg(feature = "cli")]
            let pb = spinner("Acting on goal...");
            self.act(goal.clone(), tasks, execute, max_tries).await?;
            #[cfg(feature = "cli")]
            pb.finish_with_message("Action complete!");

            #[cfg(feature = "cli")]
            let pb = spinner("Marking goal complete...");
            self.mark_goal_complete(goal);
            #[cfg(feature = "cli")]
            pb.finish_with_message("Goal marked complete!");

            #[cfg(feature = "cli")]
            let pb = spinner("Reflecting...");
            self.reflect();
            #[cfg(feature = "cli")]
            pb.finish_with_message("Reflection complete!");

            if self.has_completed_objective() {
                info!(
                    "{}",
                    format!("[*] {:?}: Objective complete!", self.agent.position())
                        .green()
                        .bold()
                );
                self.agent.update(Status::Completed);
                break;
            }
        }

        self.agent.update(Status::Idle);
        Ok(())
    }
}
