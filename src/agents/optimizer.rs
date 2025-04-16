//! # `OptimizerGPT` agent.
//!
//! This module provides functionality for managing optimization and modularization tasks
//! using the Gemini API. The `OptimizerGPT` agent is designed to help with the optimization of
//! code, refactoring existing code into smaller, more modular components. It is capable of
//! interacting with the Gemini API to generate responses, refactor code, and handle various code
//! improvement tasks in different programming languages such as Python, Rust, and JavaScript.
//!
//! # Example - Optimizing and modularizing backend code:
//!
//! ```rust
//! use autogpt::agents::optimizer::OptimizerGPT;
//! use autogpt::common::utils::Tasks;
//! use autogpt::traits::functions::Functions;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut optimizer_agent = OptimizerGPT::new(
//!         "Optimize and modularize backend code",
//!         "OptimizerGPT",
//!         "rust",
//!     );
//!
//!     let mut tasks = Tasks {
//!         description: "Refactor backend code for better modularization".into(),
//!         scope: None,
//!         urls: None,
//!         frontend_code: None,
//!         backend_code: None,
//!         api_schema: None,
//!     };
//!
//!     if let Err(err) = optimizer_agent.execute(&mut tasks, true, false, 3).await {
//!         eprintln!("Error executing optimization tasks: {:?}", err);
//!     }
//! }
//! ```
//!
//! # Key Features:
//!
//! - **Optimization and Modularization**: The `OptimizerGPT` agent focuses on improving existing code
//!   by modularizing it and refactoring large functions or classes into smaller, reusable components.
//!
//! - **Multilingual Support**: The agent supports multiple programming languages including Python, Rust,
//!   and JavaScript, adapting to the respective syntax and optimization needs.
//!
//! - **Communication with Gemini API**: It communicates with the Gemini API for generating optimized
//!   code suggestions, bug fixes, and improvements based on the provided code and prompts.
//!
//! # Methods
//!
//! - **new**: Initializes a new `OptimizerGPT` instance with the objective, position, and programming language.
//! - **save_module**: Saves the generated or optimized module to the specified workspace.
//! - **execute**: Executes the modularization and optimization task, interacting with the Gemini API to
//!   generate optimized code and modularize large codebases into smaller components.
//!
//! # Example Use Case:
//!
//! A developer wants to modularize their backend code in Rust. They use the `OptimizerGPT` to break down
//! large functions into smaller ones, improve code readability, and add necessary imports to the main file.
//! The agent interacts with the Gemini API to make recommendations, refactor the code, and save the modules
//! back to the workspace directory.

use crate::agents::agent::AgentGPT;
#[cfg(feature = "cli")]
use crate::common::utils::strip_code_blocks;
use crate::common::utils::{ClientType, Communication, Status, Tasks};
use crate::prompts::optimizer::{MODULARIZE_PROMPT, SPLIT_PROMPT};
use crate::traits::agent::Agent;
use crate::traits::functions::Functions;
use anyhow::Result;
use colored::*;
use std::borrow::Cow;
use std::env::var;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tracing::{debug, error, info};

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

/// Struct representing an `OptimizerGPT`, which manages code optimization and modularization tasks using the Gemini API.
#[derive(Debug, Clone)]
pub struct OptimizerGPT {
    /// Represents the path to the workspace directory for the backend code.
    /// This directory is where all generated or modified code is stored.
    pub workspace: Cow<'static, str>,

    /// Represents the GPT agent responsible for handling optimization and modularization tasks.
    agent: AgentGPT,

    /// Represents the programming language used in the current workspace (e.g., "python", "rust", "javascript").
    /// This helps the optimizer tailor its behavior based on the language of the code being optimized.
    pub language: String,

    /// Represents the Gemini or OpenAI client for interacting with their API.
    /// The client is used to send requests and receive responses from the Gemini model to handle optimization tasks.
    client: ClientType,
}

impl OptimizerGPT {
    /// Constructs a new instance of `OptimizerGPT`.
    ///
    /// # Arguments
    ///
    /// * `objective` - A static string describing the agent's main purpose or mission.
    /// * `position` - A static string indicating the role or position of the agent.
    /// * `language` - A string slice specifying the programming language used in the workspace (e.g., "python", "rust", "javascript").
    ///
    /// # Returns
    ///
    /// (`OptimizerGPT`): A fully initialized instance of the optimizer agent.
    ///
    /// # Behavior
    ///
    /// - Sets up a workspace directory under the `AUTOGPT_WORKSPACE` environment variable or defaults to `"workspace/backend"`.
    /// - Initializes the internal GPT agent with the given objective and position.
    /// - Creates a Gemini client using credentials pulled from environment variables (`GEMINI_API_KEY` and `GEMINI_MODEL`).
    /// - Logs status updates and prepares the environment for optimization tasks.
    ///
    /// # Business Logic
    ///
    /// - Ensures the working directory exists before continuing.
    /// - Establishes the foundational state for performing code modularization or refactoring.
    #[allow(unreachable_code)]
    #[allow(unused)]
    pub fn new(objective: &'static str, position: &'static str, language: &str) -> Self {
        let base_workspace = var("AUTOGPT_WORKSPACE").unwrap_or("workspace/".to_string());
        let workspace = format!("{}/backend", base_workspace);

        if !Path::new(&workspace).exists() {
            match fs::create_dir_all(&workspace) {
                Ok(_) => debug!("Created directory: {}", workspace),
                Err(e) => error!("Failed to create directory '{}': {}", workspace, e),
            }
        }

        let agent = AgentGPT::new_borrowed(objective, position);

        let client = ClientType::from_env();

        info!(
            "{}",
            format!("[*] {:?}: 🔧 Optimizer ready!", agent.position())
                .bright_white()
                .bold()
        );

        Self {
            workspace: workspace.into(),
            agent,
            client,
            language: language.to_string(),
        }
    }

    /// Saves a generated code module to the appropriate file in the workspace.
    ///
    /// # Arguments
    ///
    /// * `filename` - The name of the file (relative to the workspace) to which the module will be saved.
    /// * `content` - The string content of the module to be written to the file.
    ///
    /// # Returns
    ///
    /// (`Result<()>`): Returns `Ok(())` if the operation succeeds, or an error if writing fails.
    ///
    /// # Behavior
    ///
    /// - Constructs the full file path from the workspace and given filename.
    /// - Ensures that the directory structure for the target file exists (creating directories if needed).
    /// - Writes the provided content to the specified file.
    ///
    /// # Business Logic
    ///
    /// - Supports the modularization process by persisting code modules created during optimization.
    /// - Maintains a clean and structured file hierarchy within the agent’s workspace.
    pub fn save_module(&self, filename: &str, content: &str) -> Result<()> {
        let path = format!("{}/{}", self.workspace, filename);
        let parent = Path::new(&path).parent().unwrap();
        fs::create_dir_all(parent)?;

        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Asynchronously sends a prompt to the Gemini API, tracks the response, and returns the processed result.
    ///
    /// # Arguments
    ///
    /// * `request` - A string slice containing the prompt to be sent to the Gemini model.
    ///
    /// # Returns
    ///
    /// (`String`): A cleaned and formatted response string returned by Gemini. Returns an empty string if the API call fails.
    ///
    /// # Behavior
    ///
    /// - Sends a request to the Gemini API using the provided prompt.
    /// - Strips any markdown-style code block formatting (e.g., backticks) from the returned content.
    /// - Adds the Gemini response to the agent's internal communication log for traceability.
    /// - If memory is enabled (via the `mem` feature), the response is also stored in the agent's long-term memory.
    /// - In the event of an error from the Gemini API, logs and saves the error message, and returns an empty string.
    ///
    /// # Business Logic
    ///
    /// - Facilitates communication between the agent and the Gemini model.
    /// - Ensures that all model interactions are logged and optionally persisted for future context or audits.
    /// - Prepares the returned content for further downstream processing (e.g., file writing or parsing).
    #[allow(unreachable_code)]
    #[allow(unused)]
    pub async fn generate_and_track(&mut self, request: &str) -> Result<String> {
        let gemini_response = match &mut self.client {
            #[cfg(feature = "gem")]
            ClientType::Gemini(gem_client) => {
                let parameters = ChatBuilder::default()
                    .messages(vec![Message::User {
                        content: Content::Text(request.to_string()),
                        name: None,
                    }])
                    .build()?;

                let result = gem_client.chat().generate(parameters).await;

                match result {
                    Ok(response) => {
                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(response.clone()),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(response.clone()),
                                })
                                .await;
                        }

                        strip_code_blocks(&response)
                    }
                    Err(err) => {
                        let err_msg = format!("Error retrieving Gemini response: {}", err);

                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(err_msg.clone()),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(err_msg),
                                })
                                .await;
                        }

                        Default::default()
                    }
                }
            }

            #[cfg(feature = "oai")]
            ClientType::OpenAI(oai_client) => {
                let parameters = ChatCompletionParametersBuilder::default()
                    .model(FlagshipModel::Gpt4O.to_string())
                    .messages(vec![ChatMessage::User {
                        content: ChatMessageContent::Text(request.to_string()),
                        name: None,
                    }])
                    .response_format(ChatCompletionResponseFormat::Text)
                    .build()?;

                let result = oai_client.chat().create(parameters).await;

                match result {
                    Ok(chat_response) => {
                        let message = &chat_response.choices[0].message;

                        let response_text = match message {
                            ChatMessage::Assistant {
                                content: Some(chat_content),
                                ..
                            } => chat_content.to_string(),
                            ChatMessage::User { content, .. } => content.to_string(),
                            ChatMessage::System { content, .. } => content.to_string(),
                            ChatMessage::Developer { content, .. } => content.to_string(),
                            ChatMessage::Tool { content, .. } => content.clone(),
                            _ => String::from(""),
                        };

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

                        strip_code_blocks(&response_text)
                    }
                    Err(err) => {
                        let err_msg = format!("Error retrieving OpenAI response: {}", err);

                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(err_msg.clone()),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(err_msg),
                                })
                                .await;
                        }

                        Default::default()
                    }
                }
            }

            #[allow(unreachable_patterns)]
            _ => {
                let err_msg =
                    "No valid AI client configured. Enable `gem` or `oai` feature.".to_string();

                self.agent.add_communication(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(err_msg.clone()),
                });

                #[cfg(feature = "mem")]
                {
                    let _ = self
                        .save_ltm(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(err_msg),
                        })
                        .await;
                }

                Default::default()
            }
        };

        Ok(gemini_response)
    }
}
/// Implementation of the `Functions` trait for the `OptimizerGPT` struct.
///
/// This implementation provides core functionality to interact with and operate
/// the optimizer agent in a code refinement and enhancement pipeline. It defines
/// behaviors specific to the optimizer's responsibilities in the system.
///
/// # Responsibilities
///
/// - **Agent Access**: Provides a method to retrieve the internal `AgentGPT` instance
///   for logging, tracking, or user interaction.
/// - **Task Execution**: Implements an asynchronous executor for handling optimization tasks,
///   such as refactoring code, applying performance improvements, and integrating feedback
///   from previous stages in the pipeline.
///
/// # Business Logic
///
/// - Acts on tasks passed down from previous GPT roles (e.g., frontend gpt, backend gpt).
/// - Interacts with the user or system via the `AgentGPT` communication layer.
/// - Applies AI-driven code analysis and improvements.
/// - Performs logging and memory storage where applicable.
/// - Manages retry logic and ensures clean fallback/error handling.
///
/// # Notes
///
/// This trait is designed to be shared among multiple AI roles (FrontendGPT, OptimizerGPT, etc.)
/// to enforce a consistent interface for task execution across stages of the autonomous dev agent.
impl Functions for OptimizerGPT {
    /// Retrieves a reference to the agent.
    ///
    /// # Returns
    ///
    /// (`&AgentGPT`): A reference to the agent.
    ///
    fn get_agent(&self) -> &AgentGPT {
        &self.agent
    }

    /// Asynchronously executes the modularization task for a given source code file.
    ///
    /// # Arguments
    ///
    /// * `tasks` - Mutable reference to the `Tasks` struct containing task details.
    /// * `_execute` - Boolean flag indicating whether to execute the task (currently unused).
    /// * `_browse` - Boolean flag indicating whether browsing capabilities are enabled (currently unused).
    /// * `_max_tries` - Maximum number of execution attempts (currently unused).
    ///
    /// # Returns
    ///
    /// (`Result<()>`): Result indicating success or failure of the modularization process.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Reading the original code file fails.
    /// - Gemini API calls for modularization or splitting fail.
    /// - Writing to any output file fails.
    ///
    /// # Business Logic
    ///
    /// - Determines the main source code file path based on the selected language.
    /// - Reads the content of the main file and sends it to Gemini to determine a list of modular file names.
    /// - For each suggested file, sends a prompt to Gemini to generate the content of that module.
    /// - Saves each generated module in the workspace directory and logs the interaction.
    /// - Updates the original source file with appropriate import statements for the new modules.
    /// - Persists all user and assistant messages to long-term memory (if enabled).
    /// - Updates the agent's status to `Completed` after successful execution.
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
                "[*] {:?}: Executing modularization task",
                self.agent.position()
            )
            .bright_white()
            .bold()
        );

        let file_path = match self.language.as_str() {
            "python" => format!("{}/main.py", self.workspace),
            "rust" => format!("{}/src/main.rs", self.workspace),
            "javascript" => format!("{}/src/index.js", self.workspace),
            _ => panic!("Unsupported language."),
        };
        let original_code = fs::read_to_string(&file_path)?;

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(format!("Analyzing and modularizing: {}", file_path)),
        });

        #[cfg(feature = "mem")]
        self.save_ltm(Communication {
            role: Cow::Borrowed("user"),
            content: Cow::Owned("Original code sent for modularization".to_string()),
        })
        .await?;

        let prompt = format!("{}\n\n{}", MODULARIZE_PROMPT, original_code);
        let file_list_raw = self.generate_and_track(&prompt).await?;

        let filenames: Vec<String> = file_list_raw
            .lines()
            .filter(|line| {
                line.trim().ends_with(".py")
                    || line.trim().ends_with(".rs")
                    || line.trim().ends_with(".js")
            })
            .map(|line| line.trim().to_string())
            .collect();

        for filename in &filenames {
            let split_prompt = format!(
                "{}\n\nFilename: {}\nContent:\n{}",
                SPLIT_PROMPT, filename, original_code
            );
            let response = self.generate_and_track(&split_prompt).await?;

            self.save_module(filename, &response)?;

            self.agent.add_communication(Communication {
                role: Cow::Borrowed("assistant"),
                content: Cow::Owned(format!("Generated module: {}", filename)),
            });

            #[cfg(feature = "mem")]
            self.save_ltm(Communication {
                role: Cow::Borrowed("assistant"),
                content: Cow::Owned(format!("Saved file: {}", filename)),
            })
            .await?;
        }

        let imports: String = filenames
            .iter()
            .map(|f| match self.language.as_str() {
                "python" => format!("import {}", f.replace(".py", "").replace("/", ".")),
                "rust" => format!("mod {};", f.replace(".rs", "").replace("/", "::")),
                "javascript" => format!("import './{}';", f),
                _ => String::new(),
            })
            .collect::<Vec<_>>()
            .join("\n");
        if !imports.is_empty() {
            fs::write(file_path.clone(), &imports)?;
            tasks.backend_code = Some(imports.clone().into());
        }
        self.agent.update(Status::Completed);

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
    #[cfg(feature = "mem")]
    async fn save_ltm(&mut self, communication: Communication) -> Result<()> {
        save_long_term_memory(&mut self.client, self.agent.id.clone(), communication).await
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
    #[cfg(feature = "mem")]
    async fn get_ltm(&self) -> Result<Vec<Communication>> {
        load_long_term_memory(self.agent.id.clone()).await
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
    #[cfg(feature = "mem")]
    async fn ltm_context(&self) -> String {
        long_term_memory_context(self.agent.id.clone()).await
    }
}
