//! # `ArchitectGPT` agent.
//!
//! This module provides functionality for creating innovative website designs
//! and architectural diagrams based on prompts using Gemini API and diagrams library.
//! The `ArchitectGPT` agent understands user requirements and generates architectural diagrams
//! for your web applications.
//!
//! # Example - Generating website designs:
//!
//! ```rust
//! use autogpt::agents::architect::ArchitectGPT;
//! use autogpt::common::utils::Tasks;
//! use autogpt::traits::functions::Functions;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut architect_agent = ArchitectGPT::new(
//!         "Create innovative website designs",
//!         "Web wireframes and UIs",
//!     );
//!
//!     let mut tasks = Tasks {
//!         description: "Design an architectural diagram for a modern chat application".into(),
//!         scope: None,
//!         urls: None,
//!         frontend_code: None,
//!         backend_code: None,
//!         api_schema: None,
//!     };
//!
//!     if let Err(err) = architect_agent.execute(&mut tasks, true, false, 3).await {
//!         eprintln!("Error executing architect tasks: {:?}", err);
//!     }
//! }
//! ```

use crate::agents::agent::AgentGPT;
use crate::common::utils::{extract_array, extract_json_string, strip_code_blocks};
use crate::common::utils::{Communication, Scope, Status, Tasks};
use crate::prompts::architect::{
    ARCHITECT_DIAGRAM_PROMPT, ARCHITECT_ENDPOINTS_PROMPT, ARCHITECT_SCOPE_PROMPT,
};
use crate::traits::agent::Agent;
use crate::traits::functions::Functions;
use anyhow::Result;
use colored::*;
use gems::Client;
use reqwest::Client as ReqClient;
use std::borrow::Cow;
use std::env::var;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use tracing::{debug, error, info};

#[cfg(feature = "mem")]
use {
    crate::common::memory::load_long_term_memory, crate::common::memory::long_term_memory_context,
    crate::common::memory::save_long_term_memory,
};

/// Struct representing an ArchitectGPT, which orchestrates tasks related to architectural design using GPT.
#[derive(Debug, Clone)]
pub struct ArchitectGPT {
    /// Represents the workspace directory path for ArchitectGPT.
    workspace: Cow<'static, str>,
    /// Represents the GPT agent responsible for handling architectural tasks.
    agent: AgentGPT,
    /// Represents a Gemini client for interacting with Gemini API.
    client: Client,
    /// Represents a client for making HTTP requests.
    req_client: ReqClient,
}

impl ArchitectGPT {
    /// Creates a new instance of `ArchitectGPT`.
    ///
    /// # Arguments
    ///
    /// * `objective` - The objective of the agent.
    /// * `position` - The position of the agent.
    ///
    /// # Returns
    ///
    /// (`ArchitectGPT`): A new instance of ArchitectGPT.
    ///
    /// # Business Logic
    ///
    /// - Constructs the workspace directory path for ArchitectGPT.
    /// - Creates the workspace directory if it does not exist.
    /// - Initializes the GPT agent with the given objective and position.
    /// - Creates clients for interacting with Gemini API and making HTTP requests.
    ///
    pub fn new(objective: &'static str, position: &'static str) -> Self {
        let workspace = var("AUTOGPT_WORKSPACE")
            .unwrap_or("workspace/".to_string())
            .to_owned()
            + "architect";

        if !Path::new(&workspace).exists() {
            match fs::create_dir_all(workspace.clone()) {
                Ok(_) => debug!("Directory '{}' created successfully!", workspace),
                Err(e) => error!("Error creating directory '{}': {}", workspace, e),
            }
        } else {
            debug!("Directory '{}' already exists.", workspace);
        }

        match fs::write(workspace.clone() + "/diagram.py", "") {
            Ok(_) => debug!("File 'diagram.py' created successfully!"),
            Err(e) => error!("Error creating file 'diagram.py': {}", e),
        }

        let create_venv = Command::new("python3")
            .arg("-m")
            .arg("venv")
            .arg(workspace.clone() + "/.venv")
            .status();

        if let Ok(status) = create_venv {
            if status.success() {
                let pip_path = format!("{}/bin/pip", workspace.clone() + "/.venv");
                let pip_install = Command::new(pip_path)
                    .arg("install")
                    .arg("diagrams")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();

                match pip_install {
                    Ok(_) => info!(
                        "{}",
                        format!("[*] {:?}: Diagrams installed successfully!", position)
                            .bright_white()
                            .bold()
                    ),
                    Err(e) => error!(
                        "{}",
                        format!("[*] {:?}: Error installing Diagrams: {}!", position, e)
                            .bright_red()
                            .bold()
                    ),
                }
            }
        }

        let agent: AgentGPT = AgentGPT::new_borrowed(objective, position);
        let model = var("GEMINI_MODEL")
            .unwrap_or("gemini-2.0-flash".to_string())
            .to_owned();
        let api_key = var("GEMINI_API_KEY").unwrap_or_default().to_owned();
        let client = Client::new(&api_key, &model);
        info!(
            "{}",
            format!("[*] {:?}: 🛠️  Getting ready!", agent.position(),)
                .bright_white()
                .bold()
        );

        let req_client: ReqClient = ReqClient::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap();

        Self {
            workspace: workspace.into(),
            agent,
            client,
            req_client,
        }
    }

    /// Retrieves the scope based on tasks description and logs the interaction in agent memory.
    ///
    /// # Arguments
    ///
    /// * `tasks` - The tasks to be performed.
    ///
    /// # Returns
    ///
    /// (`Result<Scope>`): The scope generated based on the tasks description.
    ///
    /// # Side Effects
    ///
    /// - Updates the agent status to `Status::Completed` upon successful completion.
    /// - Adds both the user prompt and AI response (or error message) to the agent's communication memory.
    ///
    /// # Business Logic
    ///
    /// - Constructs a request based on the provided tasks.
    /// - Sends the request to the Gemini API to generate content.
    /// - Logs the request as a user communication.
    /// - Parses the response into a Scope object.
    /// - Logs the response (or error) as an assistant communication.
    /// - Updates the tasks with the retrieved scope.
    /// - Updates the agent status to `Completed`.
    pub async fn get_scope(&mut self, tasks: &mut Tasks) -> Result<Scope> {
        let request: String = format!(
            "{}\n\nHere is the User Request:{}",
            ARCHITECT_SCOPE_PROMPT, tasks.description
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

        let gemini_response_result = self.client.generate_content(&request).await;

        let gemini_response = match gemini_response_result {
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

                serde_json::from_str(&extract_json_string(&response).unwrap_or_default())?
            }
            Err(err) => {
                error!("[*] {:?}: {:?}", self.agent.position(), err);

                self.agent.add_communication(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(format!("Error generating content: {}", err)),
                });
                #[cfg(feature = "mem")]
                {
                    let _ = self
                        .save_ltm(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(format!("Error generating content: {}", err)),
                        })
                        .await;
                }

                Default::default()
            }
        };

        tasks.scope = Some(gemini_response);
        self.agent.update(Status::Completed);
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }

    /// Retrieves URLs based on tasks description and logs the interaction in agent memory.
    ///
    /// # Arguments
    ///
    /// * `tasks` - The tasks to be performed.
    ///
    /// # Returns
    ///
    /// (`Result<()>`): Result indicating success or failure of the operation.
    ///
    /// # Side Effects
    ///
    /// - Updates the agent status to `Status::InUnitTesting` upon successful completion.
    /// - Adds both the user prompt and AI response (or error message) to the agent's communication memory.
    ///
    /// # Business Logic
    ///
    /// - Constructs a request based on the provided tasks.
    /// - Sends the request to the GPT client to generate content.
    /// - Logs the request as a user communication.
    /// - Parses the response into a vector of URLs.
    /// - Logs the response (or error) as an assistant communication.
    /// - Updates the tasks with the retrieved URLs.
    /// - Updates the agent status to `InUnitTesting`.
    pub async fn get_urls(&mut self, tasks: &mut Tasks) -> Result<()> {
        let request: String = format!(
            "{}\n\nHere is the Project Description:{}",
            ARCHITECT_ENDPOINTS_PROMPT, tasks.description
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

        let gemini_response_result = self.client.generate_content(&request).await;

        let gemini_response: Vec<Cow<'static, str>> = match gemini_response_result {
            Ok(response) => {
                debug!(
                    "[*] {:?}: Got Response {:?}",
                    self.agent.position(),
                    response
                );

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

                serde_json::from_str(&extract_array(&response).unwrap_or_default())?
            }
            Err(err) => {
                error!("[*] {:?}: {:?}", self.agent.position(), err);

                self.agent.add_communication(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(format!("Error getting URLs: {}", err)),
                });
                #[cfg(feature = "mem")]
                {
                    let _ = self
                        .save_ltm(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(format!("Error getting URLs: {}", err)),
                        })
                        .await;
                }

                Default::default()
            }
        };

        tasks.urls = Some(gemini_response);
        self.agent.update(Status::InUnitTesting);
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(())
    }

    /// Generates a diagram based on tasks description and logs the interaction in agent memory.
    ///
    /// # Arguments
    ///
    /// * `tasks` - The tasks to be performed.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): The generated diagram content.
    ///
    /// # Side Effects
    ///
    /// - Adds both the user prompt and AI response (or error message) to the agent's communication memory.
    ///
    /// # Business Logic
    ///
    /// - Constructs a request based on the provided tasks.
    /// - Sends the request to the GPT client to generate content.
    /// - Logs the request as a user communication.
    /// - Logs the response (or error) as an assistant communication.
    /// - Processes the response to strip code blocks.
    /// - Returns the cleaned-up diagram content.
    pub async fn generate_diagram(&mut self, tasks: &mut Tasks) -> Result<String> {
        let request: String = format!(
            "{}\n\nUser Request:{}",
            ARCHITECT_DIAGRAM_PROMPT, tasks.description
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

        let gemini_response_result = self.client.generate_content(&request).await;

        let gemini_response = match gemini_response_result {
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
                self.agent.add_communication(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(format!("Error generating diagram: {}", err)),
                });

                #[cfg(feature = "mem")]
                {
                    let _ = self
                        .save_ltm(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(format!("Error generating diagram: {}", err)),
                        })
                        .await;
                }

                Default::default()
            }
        };

        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }

    /// Retrieves a reference to the agent.
    ///
    /// # Returns
    ///
    /// (`&AgentGPT`): A reference to the agent.
    ///
    /// # Business Logic
    /// - Provides access to the agent associated with the ArchitectGPT instance.
    ///
    pub fn agent(&self) -> &AgentGPT {
        &self.agent
    }
}

/// Implementation of the trait `Functions` for `ArchitectGPT`.
/// Contains additional methods related to architectural tasks.
///
/// This trait provides methods for:
/// - Retrieving the agent associated with `ArchitectGPT`.
/// - Executing tasks asynchronously.
///
/// # Business Logic
///
/// - Provides access to the agent associated with the `ArchitectGPT` instance.
/// - Executes tasks asynchronously based on the current status of the agent.
/// - Handles task execution including scope retrieval, URL retrieval, and diagram generation.
/// - Manages retries in case of failures during task execution.
///
impl Functions for ArchitectGPT {
    /// Retrieves a reference to the agent.
    ///
    /// # Returns
    ///
    /// (`&AgentGPT`): A reference to the agent.
    ///
    fn get_agent(&self) -> &AgentGPT {
        &self.agent
    }

    /// Executes tasks asynchronously.
    ///
    /// # Arguments
    ///
    /// * `tasks` - The tasks to be executed.
    /// * `execute` - Flag indicating whether to execute the tasks.
    /// * `max_tries` - Maximum number of retry attempts.
    ///
    /// # Returns
    ///
    /// (`Result<()>`): Result indicating success or failure of the operation.
    ///
    /// # Business Logic
    ///
    /// - Executes tasks asynchronously based on the current status of the agent.
    /// - Handles task execution including scope retrieval, URL retrieval, and diagram generation.
    /// - Manages retries in case of failures during task execution.
    ///
    async fn execute(
        &mut self,
        tasks: &mut Tasks,
        _execute: bool,
        _browse: bool,
        max_tries: u64,
    ) -> Result<()> {
        info!(
            "{}",
            format!("[*] {:?}: Executing task:", self.agent.position(),)
                .bright_white()
                .bold()
        );
        for task in tasks.clone().description.clone().split("- ") {
            if !task.trim().is_empty() {
                info!("{} {}", "•".bright_white().bold(), task.trim().cyan());
            }
        }

        let path = &(self.workspace.to_string() + "/diagram.py");

        while self.agent.status() != &Status::Completed {
            match self.agent.status() {
                Status::Idle => {
                    debug!("[*] {:?}: Idle", self.agent.position());

                    let scope: Scope = self.get_scope(tasks).await?;

                    if scope.external {
                        let _ = self.get_urls(tasks).await;
                    }
                    self.agent.update(Status::InUnitTesting);
                }

                Status::InUnitTesting => {
                    let mut exclude: Vec<Cow<'static, str>> = Vec::new();

                    let urls: &Vec<Cow<'static, str>> = &tasks
                        .urls
                        .as_ref()
                        .map_or_else(Vec::default, |url| url.to_vec());

                    for url in urls {
                        info!(
                            "{}",
                            format!(
                                "[*] {:?}: Testing URL Endpoint: {}",
                                self.agent.position(),
                                url
                            )
                            .bright_white()
                            .bold()
                        );

                        // ping url
                        let status_code_result = self.req_client.get(url.to_string()).send().await;

                        match status_code_result {
                            Ok(response) => {
                                let status_code = response.status();
                                if status_code != reqwest::StatusCode::OK {
                                    exclude.push(url.clone());
                                }
                            }
                            Err(err) => {
                                let url = err
                                    .url()
                                    .map(|u| u.to_string())
                                    .unwrap_or_else(|| "unknown URL".to_string());
                                let msg = format!(
                                    " Failed to retrieve data from '{}'. Please check the URL or your internet connection.",
                                    url
                                );
                                error!(
                                    "{}",
                                    format!(
                                        "[*] {:?}: Error sending request for URL {}: {:?}",
                                        self.agent.position(),
                                        url,
                                        msg
                                    )
                                    .bright_red()
                                    .bold()
                                );
                            }
                        }
                    }

                    // remove link rot
                    if !exclude.is_empty() {
                        let new_urls: Vec<Cow<'static, str>> = tasks
                            .urls
                            .as_ref()
                            .unwrap()
                            .iter()
                            .filter(|url| !exclude.contains(url))
                            .cloned()
                            .collect();
                        tasks.urls = Some(new_urls);
                    }

                    // generate an architectural diagram
                    // Require root: install necessary deps
                    // let graphviz_install = Command::new("sudo")
                    //     .arg("apt-get")
                    //     .arg("install")
                    //     .arg("graphviz")
                    //     .spawn();

                    // match graphviz_install {
                    //     Ok(_) => debug!("Graphviz installed successfully!"),
                    //     Err(e) => error!("Error installing Graphviz: {}", e),
                    // }

                    let python_code = self.generate_diagram(tasks).await?;

                    // Write the content to the file
                    match fs::write(path, python_code.clone()) {
                        Ok(_) => debug!("File 'diagram.py' created successfully!"),
                        Err(e) => error!("Error creating file 'diagram.py': {}", e),
                    }

                    for attempt in 1..=max_tries {
                        let run_python = Command::new("timeout")
                            .arg(format!("{}s", 10))
                            .arg("python3")
                            .arg(path)
                            .current_dir(self.workspace.to_string())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn();

                        match run_python {
                            Ok(_) => {
                                info!("Diagram generated successfully!");
                                break;
                            }
                            Err(e) => {
                                error!("Error generating the diagram: {}", e);
                                if attempt < max_tries {
                                    info!("Retrying...");
                                    tasks.description = (tasks.description.to_string()
                                        + "Got an error: "
                                        + &e.to_string()
                                        + "while running code: "
                                        + &python_code.clone())
                                        .into();
                                    let python_code = self.generate_diagram(tasks).await?;

                                    match fs::write(path, python_code.clone()) {
                                        Ok(_) => debug!("File 'diagram.py' created successfully!"),
                                        Err(e) => error!("Error creating file 'diagram.py': {}", e),
                                    }
                                } else {
                                    info!("Maximum retries reached, exiting...");
                                    break;
                                }
                            }
                        }
                    }

                    self.agent.update(Status::Completed);
                }

                _ => {
                    self.agent.update(Status::Completed);
                }
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
