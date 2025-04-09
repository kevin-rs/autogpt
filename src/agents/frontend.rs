//! # `FrontendGPT` agent.
//!
//! This module provides functionality for generating frontend code based on prompts
//! using Gemini API. The `FrontendGPT` agent is capable of understanding user requests
//! and producing code snippets in various programming languages and frameworks commonly
//! used for web development.
//!
//! # Example - Generating frontend code:
//!
//! ```rust
//! use autogpt::agents::frontend::FrontendGPT;
//! use autogpt::common::utils::Tasks;
//! use autogpt::traits::functions::Functions;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut frontend_agent = FrontendGPT::new(
//!         "Generate frontend code",
//!         "Frontend Developer",
//!         "rust",
//!     );
//!
//!     let mut tasks = Tasks {
//!         description: "Create a landing page with a sign-up form".into(),
//!         scope: None,
//!         urls: None,
//!         frontend_code: None,
//!         backend_code: None,
//!         api_schema: None,
//!     };
//!
//!     if let Err(err) = frontend_agent.execute(&mut tasks, true, false, 3).await {
//!         eprintln!("Error executing frontend tasks: {:?}", err);
//!     }
//! }
//! ```
//!

use crate::agents::agent::AgentGPT;
use crate::common::utils::{strip_code_blocks, ClientType, Communication, Status, Tasks};
use crate::prompts::frontend::{
    FIX_CODE_PROMPT, FRONTEND_CODE_PROMPT, IMPROVED_FRONTEND_CODE_PROMPT,
};
use crate::traits::agent::Agent;
use crate::traits::functions::Functions;
use anyhow::Result;
use colored::*;
use reqwest::Client as ReqClient;
use std::borrow::Cow;
use std::env::var;
use std::fs;
use std::io::Read;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use tracing::{debug, error, info, warn};

#[cfg(feature = "mem")]
use {
    crate::common::memory::load_long_term_memory, crate::common::memory::long_term_memory_context,
    crate::common::memory::save_long_term_memory,
};
#[cfg(feature = "oai")]
use {openai_dive::v1::models::FlagshipModel, openai_dive::v1::resources::chat::*};

/// Struct representing a `FrontendGPT`, which manages frontend code generation and testing using Gemini API.
#[derive(Debug, Clone)]
#[allow(unused)]
pub struct FrontendGPT {
    /// Represents the workspace directory path for `FrontendGPT`.
    workspace: Cow<'static, str>,
    /// Represents the GPT agent responsible for handling frontend tasks.
    agent: AgentGPT,
    /// Represents an OpenAI or Gemini client for interacting with their API.
    client: ClientType,
    /// Represents a client for sending HTTP requests.
    req_client: ReqClient,
    /// Represents the bugs found in the code.
    bugs: Option<Cow<'static, str>>,
    /// Represents the programming language used for frontend development.
    language: &'static str,
    /// Represents the number of bugs found in the code.
    nb_bugs: u64,
}

impl FrontendGPT {
    /// Constructor function to create a new instance of FrontendGPT.
    ///
    /// # Arguments
    ///
    /// * `objective` - Objective description for FrontendGPT.
    /// * `position` - Position description for FrontendGPT.
    /// * `language` - Programming language used for frontend development.
    ///
    /// # Returns
    ///
    /// (`FrontendGPT`): A new instance of FrontendGPT.
    ///
    /// # Business Logic
    ///
    /// - Constructs the workspace directory path for FrontendGPT.
    /// - Initializes the GPT agent with the given objective, position, and language.
    /// - Creates clients for interacting with Gemini API
    ///
    pub fn new(objective: &'static str, position: &'static str, language: &'static str) -> Self {
        let workspace = var("AUTOGPT_WORKSPACE")
            .unwrap_or("workspace/".to_string())
            .to_owned()
            + "frontend";

        if let Err(e) = fs::create_dir_all(workspace.clone()) {
            error!("Error creating directory '{}': {}", workspace, e);
        }

        match language {
            "rust" => {
                let cargo_new = Command::new("cargo")
                    .arg("init")
                    .arg(workspace.clone())
                    .spawn();

                match cargo_new {
                    Ok(_) => debug!("Cargo project initialized successfully!"),
                    Err(e) => error!("Error initializing Cargo project: {}", e),
                }
                match fs::write(workspace.clone() + "/src/template.rs", "") {
                    Ok(_) => debug!("File 'template.rs' created successfully!"),
                    Err(e) => error!("Error creating file 'template.rs': {}", e),
                };
            }
            "python" => {
                match fs::write(workspace.clone() + "/main.py", "") {
                    Ok(_) => debug!("File 'main.py' created successfully!"),
                    Err(e) => error!("Error creating file 'main.py': {}", e),
                }
                match fs::write(workspace.clone() + "/template.py", "") {
                    Ok(_) => debug!("File 'template.py' created successfully!"),
                    Err(e) => error!("Error creating file 'template.py': {}", e),
                };
            }
            "javascript" => {
                let npx_install = Command::new("npx")
                    .arg("create-react-app")
                    .arg(workspace.clone())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn();

                match npx_install {
                    Ok(mut child) => match child.wait() {
                        Ok(status) => {
                            if status.success() {
                                debug!("React JS project initialized successfully!");
                            } else {
                                error!("Failed to initialize React JS project");
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
                match fs::write(workspace.clone() + "/src/template.js", "") {
                    Ok(_) => debug!("File 'template.js' created successfully!"),
                    Err(e) => error!("Error creating file 'template.js': {}", e),
                };
            }
            _ => panic!("Unsupported language, consider open an Issue/PR"),
        };
        let agent: AgentGPT = AgentGPT::new_borrowed(objective, position);

        let client = ClientType::from_env();

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
            bugs: None,
            language,
            nb_bugs: 0,
        }
    }

    /// Asynchronously generates frontend code based on tasks.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to tasks containing the project description.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): Result containing the generated frontend code.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in generating frontend code.
    ///
    /// # Business Logic
    ///
    /// - Determines the file path based on the programming language.
    /// - Reads the template code from the specified file.
    /// - Logs communications throughout the code generation process.
    /// - Constructs a request for generating frontend code using the template and project description.
    /// - Sends the request to the Gemini API to generate frontend code.
    /// - Writes the generated frontend code to the appropriate file.
    pub async fn generate_frontend_code(&mut self, tasks: &mut Tasks) -> Result<String> {
        let path = self.workspace.clone();

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(format!(
                "Request to generate frontend code for project: {}",
                tasks.description
            )),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("user"),
                    content: Cow::Owned(format!(
                        "Request to generate frontend code for project: {}",
                        tasks.description
                    )),
                })
                .await;
        }
        let full_path = match self.language {
            "rust" => format!("{}/{}", path, "src/template.rs"),
            "python" => format!("{}/{}", path, "template.py"),
            "javascript" => format!("{}/{}", path, "src/template.js"),
            _ => panic!("Unsupported language, consider open an Issue/PR"),
        };

        debug!("[*] {:?}: {:?}", self.agent.position(), full_path);

        let template = fs::read_to_string(&full_path)?;

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(format!(
                "Generating frontend code using template from '{}' and project description.",
                full_path
            )),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(format!(
                    "Generating frontend code using template from '{}' and project description.",
                    full_path
                )),
                })
                .await;
        }
        let request = format!(
            "{}\n\nCode Template: {}\nProject Description: {}",
            FRONTEND_CODE_PROMPT, template, tasks.description
        );

        let gemini_response = match &mut self.client {
            #[cfg(feature = "gem")]
            ClientType::Gemini(ref mut gem_client) => {
                let result = gem_client.generate_content(&request).await;

                match result {
                    Ok(response) => strip_code_blocks(&response),
                    Err(_err) => {
                        let error_msg = "Failed to generate content from Gemini API.".to_string();
                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("system"),
                            content: Cow::Owned(error_msg.clone()),
                        });
                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("system"),
                                    content: Cow::Owned(error_msg.clone()),
                                })
                                .await;
                        }
                        return Err(anyhow::anyhow!(error_msg));
                    }
                }
            }

            #[cfg(feature = "oai")]
            ClientType::OpenAI(oai_client) => {
                let parameters = ChatCompletionParametersBuilder::default()
                    .model(FlagshipModel::Gpt4O.to_string())
                    .messages(vec![ChatMessage::User {
                        content: ChatMessageContent::Text(request.clone()),
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

                        strip_code_blocks(&response_text)
                    }

                    Err(_err) => {
                        let error_msg = "Failed to generate content from OpenAI API.".to_string();
                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("system"),
                            content: Cow::Owned(error_msg.clone()),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("system"),
                                    content: Cow::Owned(error_msg.clone()),
                                })
                                .await;
                        }
                        return Err(anyhow::anyhow!(error_msg));
                    }
                }
            }

            #[allow(unreachable_patterns)]
            _ => {
                return Err(anyhow::anyhow!(
                    "No valid AI client configured. Enable `gem` or `oai` feature."
                ));
            }
        };

        let frontend_path = match self.language {
            "rust" => format!("{}/{}", path, "src/main.rs"),
            "python" => format!("{}/{}", path, "main.py"),
            "javascript" => format!("{}/{}", path, "src/index.js"),
            _ => panic!("Unsupported language, consider opening an Issue/PR"),
        };

        fs::write(&frontend_path, gemini_response.clone())?;

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(format!(
                "Frontend code generated and saved to '{}'",
                frontend_path
            )),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(format!(
                        "Frontend code generated and saved to '{}'",
                        frontend_path
                    )),
                })
                .await;
        }

        tasks.frontend_code = Some(gemini_response.clone().into());

        self.agent.update(Status::Completed);
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }
    /// Asynchronously improves existing frontend code based on tasks.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to tasks containing the project description and existing code.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): Result containing the improved frontend code.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in improving frontend code.
    ///
    /// # Business Logic
    ///
    /// - Constructs a request for improving existing frontend code using project description and current code.
    /// - Logs communication entries for tracing user intent and AI response.
    /// - Sends the request to the Gemini API to improve the frontend code.
    /// - Writes the improved frontend code to the appropriate file.
    pub async fn improve_frontend_code(&mut self, tasks: &mut Tasks) -> Result<String> {
        let path = self.workspace.clone();

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(format!(
                "Request to improve frontend code for project: {}",
                tasks.description
            )),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("user"),
                    content: Cow::Owned(format!(
                        "Request to improve frontend code for project: {}",
                        tasks.description
                    )),
                })
                .await;
        }
        let existing_code = tasks.clone().frontend_code.unwrap_or_default();

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(
                "Improving existing frontend code using project description...".to_string(),
            ),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(
                        "Improving existing frontend code using project description...".to_string(),
                    ),
                })
                .await;
        }
        let request = format!(
            "{}\n\nCode Template: {}\nProject Description: {}",
            IMPROVED_FRONTEND_CODE_PROMPT, existing_code, tasks.description
        );

        let gemini_response = match &mut self.client {
            #[cfg(feature = "gem")]
            ClientType::Gemini(ref mut gem_client) => {
                let result = gem_client.generate_content(&request).await;

                match result {
                    Ok(response) => strip_code_blocks(&response),
                    Err(_err) => {
                        let error_msg =
                            "Failed to generate improved frontend code via Gemini API.".to_string();
                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("system"),
                            content: Cow::Owned(error_msg.clone()),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("system"),
                                    content: Cow::Owned(error_msg.clone()),
                                })
                                .await;
                        }

                        return Err(anyhow::anyhow!(error_msg));
                    }
                }
            }

            #[cfg(feature = "oai")]
            ClientType::OpenAI(oai_client) => {
                let parameters = ChatCompletionParametersBuilder::default()
                    .model(FlagshipModel::Gpt4O.to_string())
                    .messages(vec![ChatMessage::User {
                        content: ChatMessageContent::Text(request.clone()),
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

                        strip_code_blocks(&response_text)
                    }

                    Err(_err) => {
                        let error_msg =
                            "Failed to generate improved frontend code via OpenAI API.".to_string();
                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("system"),
                            content: Cow::Owned(error_msg.clone()),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("system"),
                                    content: Cow::Owned(error_msg.clone()),
                                })
                                .await;
                        }

                        return Err(anyhow::anyhow!(error_msg));
                    }
                }
            }

            #[allow(unreachable_patterns)]
            _ => {
                return Err(anyhow::anyhow!(
                    "No valid AI client configured. Enable `gem` or `oai` feature."
                ));
            }
        };

        let frontend_path = match self.language {
            "rust" => format!("{}/{}", path, "src/main.rs"),
            "python" => format!("{}/{}", path, "main.py"),
            "javascript" => format!("{}/{}", path, "src/index.js"),
            _ => panic!("Unsupported language, consider opening an Issue/PR"),
        };

        fs::write(&frontend_path, gemini_response.clone())?;

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(format!(
                "Improved frontend code saved to '{}'",
                frontend_path
            )),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(format!(
                        "Improved frontend code saved to '{}'",
                        frontend_path
                    )),
                })
                .await;
        }

        tasks.frontend_code = Some(gemini_response.clone().into());

        self.agent.update(Status::Completed);
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }
    /// Asynchronously fixes bugs in the frontend code based on tasks.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to tasks containing the project description and existing code.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): Result containing the fixed frontend code.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in fixing bugs in the frontend code.
    ///
    /// # Business Logic
    ///
    /// - Constructs a request for fixing bugs in the frontend code using project description and existing code.
    /// - Logs communications throughout the process.
    /// - Sends the request to the Gemini API to fix bugs in the frontend code.
    /// - Writes the fixed frontend code to the appropriate file.
    pub async fn fix_code_bugs(&mut self, tasks: &mut Tasks) -> Result<String> {
        let path = self.workspace.clone();

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(format!(
                "Request to fix bugs in frontend code. Known bugs: {}",
                self.bugs
                    .clone()
                    .unwrap_or_else(|| "No bug description provided.".into())
            )),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("user"),
                    content: Cow::Owned(format!(
                        "Request to fix bugs in frontend code. Known bugs: {}",
                        self.bugs
                            .clone()
                            .unwrap_or_else(|| "No bug description provided.".into())
                    )),
                })
                .await;
        }
        let request = format!(
            "{}\n\nBuggy Code: {}\nBugs: {}\n\nFix all bugs.",
            FIX_CODE_PROMPT,
            tasks.clone().frontend_code.unwrap_or_default(),
            self.bugs.clone().unwrap_or_default()
        );

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(
                "Attempting to fix bugs in the provided frontend code...".to_string(),
            ),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(
                        "Attempting to fix bugs in the provided frontend code...".to_string(),
                    ),
                })
                .await;
        }

        let gemini_response = match &mut self.client {
            #[cfg(feature = "gem")]
            ClientType::Gemini(ref mut gem_client) => {
                let result = gem_client.generate_content(&request).await;

                match result {
                    Ok(response) => strip_code_blocks(&response),
                    Err(_err) => {
                        let error_msg =
                            "Failed to generate bug-fixed frontend code via Gemini API."
                                .to_string();
                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("system"),
                            content: Cow::Owned(error_msg.clone()),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("system"),
                                    content: Cow::Owned(error_msg.clone()),
                                })
                                .await;
                        }

                        return Err(anyhow::anyhow!(error_msg));
                    }
                }
            }

            #[cfg(feature = "oai")]
            ClientType::OpenAI(oai_client) => {
                let parameters = ChatCompletionParametersBuilder::default()
                    .model(FlagshipModel::Gpt4O.to_string())
                    .messages(vec![ChatMessage::User {
                        content: ChatMessageContent::Text(request.clone()),
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

                        strip_code_blocks(&response_text)
                    }

                    Err(_err) => {
                        let error_msg =
                            "Failed to generate bug-fixed frontend code via OpenAI API."
                                .to_string();
                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("system"),
                            content: Cow::Owned(error_msg.clone()),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("system"),
                                    content: Cow::Owned(error_msg.clone()),
                                })
                                .await;
                        }

                        return Err(anyhow::anyhow!(error_msg));
                    }
                }
            }

            #[allow(unreachable_patterns)]
            _ => {
                return Err(anyhow::anyhow!(
                    "No valid AI client configured. Enable `gem` or `oai` feature."
                ));
            }
        };

        let frontend_path = match self.language {
            "rust" => format!("{}/{}", path, "src/main.rs"),
            "python" => format!("{}/{}", path, "main.py"),
            "javascript" => format!("{}/{}", path, "src/index.js"),
            _ => panic!("Unsupported language, consider opening an Issue/PR"),
        };

        fs::write(&frontend_path, gemini_response.clone())?;

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(format!(
                "Bugs fixed. Updated code saved to '{}'",
                frontend_path
            )),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(format!(
                        "Bugs fixed. Updated code saved to '{}'",
                        frontend_path
                    )),
                })
                .await;
        }

        tasks.frontend_code = Some(gemini_response.clone().into());

        self.agent.update(Status::Completed);
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }

    /// Retrieves the GPT agent associated with FrontendGPT.
    ///
    /// # Returns
    ///
    /// (`&AgentGPT`): A reference to the GPT agent.
    ///
    /// # Business Logic
    ///
    /// - Provides access to the GPT agent associated with the FrontendGPT instance.
    ///
    pub fn agent(&self) -> &AgentGPT {
        &self.agent
    }

    /// Updates the bugs found in the frontend code.
    ///
    /// # Arguments
    ///
    /// * `bugs` - Option containing the bugs found in the code.
    ///
    /// # Business Logic
    ///
    /// - Updates the bugs found in the frontend code.
    ///
    pub fn update_bugs(&mut self, bugs: Option<Cow<'static, str>>) {
        self.bugs = bugs;
    }
}

/// Implementation of the trait Functions for FrontendGPT.
/// Contains additional methods related to frontend tasks.
///
/// This trait provides methods for:
///
/// - Retrieving the GPT agent associated with FrontendGPT.
/// - Executing frontend tasks asynchronously.
///
/// # Business Logic
///
/// - Provides access to the GPT agent associated with the FrontendGPT instance.
/// - Executes frontend tasks asynchronously based on the current status of the agent.
/// - Handles task execution including code generation, improvement, bug fixing, and testing.
/// - Manages retries and error handling during task execution.
///
impl Functions for FrontendGPT {
    /// Retrieves a reference to the agent.
    ///
    /// # Returns
    ///
    /// (`&AgentGPT`): A reference to the agent.
    ///
    fn get_agent(&self) -> &AgentGPT {
        &self.agent
    }

    /// Asynchronously executes frontend tasks associated with FrontendGPT.
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
    /// - Executes frontend tasks asynchronously based on the current status of the agent.
    /// - Handles task execution including code generation, improvement, bug fixing, and testing.
    /// - Manages retries and error handling during task execution.
    ///
    async fn execute(
        &mut self,
        tasks: &mut Tasks,
        execute: bool,
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

        let path = &self.workspace.to_string();

        while self.agent.status() != &Status::Completed {
            match &self.agent.status() {
                Status::Idle => {
                    let _ = self.generate_frontend_code(tasks).await;
                    self.agent.update(Status::Active);
                    continue;
                }

                Status::Active => {
                    if self.nb_bugs == 0 {
                        let _ = self.improve_frontend_code(tasks).await;
                    } else {
                        let _ = self.fix_code_bugs(tasks).await;
                    }
                    self.agent.update(Status::InUnitTesting);
                    continue;
                }

                Status::InUnitTesting => {
                    info!(
                        "{}",
                        format!(
                            "[*] {:?}: Frontend Code Unit Testing...",
                            self.agent.position(),
                        )
                        .bright_white()
                        .bold()
                    );

                    if !execute {
                        warn!(
                            "{}",
                            format!(
                                "[*] {:?}: It seems the code isn't safe to proceed...",
                                self.agent.position(),
                            )
                            .bright_yellow()
                            .bold()
                        );
                    }

                    info!(
                        "{}",
                        format!(
                            "[*] {:?}: Building and running the frontend project...",
                            self.agent.position(),
                        )
                        .bright_white()
                        .bold()
                    );

                    let result = match self.language {
                        "rust" => {
                            let mut build_command = Command::new("timeout");
                            build_command
                                .arg(format!("{}s", 10))
                                .arg("cargo")
                                .arg("build")
                                .arg("--release")
                                .current_dir(path)
                                .stdout(Stdio::piped())
                                .stderr(Stdio::piped())
                                .spawn()
                        }
                        "python" => Command::new("timeout")
                            .arg(format!("{}s", 10))
                            .arg("uvicorn")
                            .arg("main:app")
                            .current_dir(path)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn(),
                        "javascript" => Command::new("timeout")
                            .arg(format!("{}s", 10))
                            .arg("npm")
                            .arg("run")
                            .arg("build")
                            .current_dir(path)
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn(),
                        _ => panic!("Unsupported language, consider opening an Issue/PR"),
                    };

                    match result {
                        Ok(mut child) => {
                            self.nb_bugs += 1;
                            let mut stderr_output = String::new();
                            child
                                .stderr
                                .as_mut()
                                .expect("Failed to capture build stderr")
                                .read_to_string(&mut stderr_output)
                                .expect("Failed to read build stderr");
                            if self.nb_bugs > max_tries {
                                error!(
                                    "{}",
                                    format!(
                                        "[*] {:?}: Too many bugs found. Consider debugging...",
                                        self.agent.position(),
                                    )
                                    .bright_red()
                                    .bold()
                                );
                                break;
                            } else {
                                self.agent.update(Status::Active);
                            }
                            if !stderr_output.trim().is_empty() {
                                self.bugs = Some(stderr_output.into());
                            } else {
                                info!(
                                    "{}",
                                    format!(
                                        "[*] {:?}: Frontend build successful...",
                                        self.agent.position(),
                                    )
                                    .bright_green()
                                    .bold()
                                );
                            }
                        }
                        Err(err) => {
                            error!(
                                "{}",
                                format!(
                                    "[*] {:?}: Failed to execute command: {}",
                                    self.agent.position(),
                                    err
                                )
                                .bright_red()
                                .bold()
                            );
                            panic!();
                        }
                    }
                }
                _ => {}
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
