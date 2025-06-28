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
//! use autogpt::common::utils::Task;
//! use autogpt::traits::functions::Functions;
//! use autogpt::traits::functions::AsyncFunctions;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut architect_agent = ArchitectGPT::new(
//!         "Create innovative website designs",
//!         "Web wireframes and UIs",
//!     ).await;
//!
//!     let mut tasks = Task {
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
#![allow(unreachable_code)]

use crate::agents::agent::AgentGPT;
#[allow(unused_imports)]
use crate::common::utils::{
    Capability, ClientType, Communication, ContextManager, Knowledge, Persona, Planner, Reflection,
    Scope, Status, Task, TaskScheduler, Tool, extract_array, extract_json_string,
    strip_code_blocks,
};
use crate::prompts::architect::{
    ARCHITECT_DIAGRAM_PROMPT, ARCHITECT_ENDPOINTS_PROMPT, ARCHITECT_SCOPE_PROMPT,
};
use crate::traits::agent::Agent;
use crate::traits::composite::AgentFunctions;
use crate::traits::functions::{AgentExecutor, AsyncFunctions, Functions};
use anyhow::Result;
use async_trait::async_trait;
use colored::*;
use reqwest::Client as ReqClient;
use std::borrow::Cow;
use std::env::var;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::process::Command;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

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

use auto_derive::Auto;

/// Struct representing an ArchitectGPT, which orchestrates tasks related to architectural design using GPT.
#[derive(Debug, Clone, Default, Auto)]
pub struct ArchitectGPT {
    /// Represents the workspace directory path for ArchitectGPT.
    workspace: Cow<'static, str>,
    /// Represents the GPT agent responsible for handling architectural tasks.
    agent: AgentGPT,
    /// Represents a client for interacting with an AI API (OpenAI or Gemini).
    client: ClientType,
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
    /// - Creates clients for interacting with Gemini or OpenAI API and making HTTP requests.
    #[allow(unreachable_code)]
    #[allow(unused)]
    pub async fn new(objective: &'static str, position: &'static str) -> Self {
        let workspace = var("AUTOGPT_WORKSPACE")
            .unwrap_or("workspace/".to_string())
            .to_owned()
            + "architect";

        if !fs::try_exists(&workspace).await.unwrap_or(false) {
            match fs::create_dir_all(&workspace).await {
                Ok(_) => debug!("Directory '{}' created successfully!", workspace),
                Err(e) => error!("Error creating directory '{}': {}", workspace, e),
            }
        } else {
            debug!("Directory '{}' already exists.", workspace);
        }

        match fs::write(workspace.clone() + "/diagram.py", "").await {
            Ok(_) => debug!("File 'diagram.py' created successfully!"),
            Err(e) => error!("Error creating file 'diagram.py': {}", e),
        }

        let create_venv = Command::new("python3")
            .arg("-m")
            .arg("venv")
            .arg(workspace.clone() + "/.venv")
            .status();

        if let Ok(status) = create_venv.await {
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

        let client = ClientType::from_env();

        info!(
            "{}",
            format!("[*] {:?}: 🛠️  Getting ready!", agent.position())
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
    /// - Sends the request to the Gemini or OpenAI API to generate content.
    /// - Logs the request as a user communication.
    /// - Parses the response into a Scope object.
    /// - Logs the response (or error) as an assistant communication.
    /// - Updates the tasks with the retrieved scope.
    /// - Updates the agent status to `Completed`.
    #[allow(unreachable_code)]
    #[allow(unused)]
    pub async fn get_scope(&mut self, tasks: &mut Task) -> Result<Scope> {
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

        let response: Scope = match &mut self.client {
            #[cfg(feature = "gem")]
            ClientType::Gemini(gem_client) => {
                let parameters = ChatBuilder::default()
                    .messages(vec![Message::User {
                        content: Content::Text(request),
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
                                    content: Cow::Owned(format!(
                                        "Error generating content: {}",
                                        err
                                    )),
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
                        content: ChatMessageContent::Text(request),
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

                        serde_json::from_str(
                            &extract_json_string(&response_text).unwrap_or_default(),
                        )?
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
                                    content: Cow::Owned(format!(
                                        "Error generating content: {}",
                                        err
                                    )),
                                })
                                .await;
                        }

                        Default::default()
                    }
                }
            }

            #[cfg(feature = "cld")]
            ClientType::Anthropic(client) => {
                let body = CreateMessageParams::new(RequiredMessageParams {
                    model: "claude-3-7-sonnet-latest".to_string(),
                    messages: vec![AnthMessage::new_text(Role::User, request.clone())],
                    max_tokens: 1024,
                });

                match client.create_message(Some(&body)).await {
                    Ok(chat_response) => {
                        let response_text = chat_response
                            .content
                            .iter()
                            .map(|block| match block {
                                ContentBlock::Text { text, .. } => text.as_str(),
                                _ => "",
                            })
                            .collect::<Vec<_>>()
                            .join("\n");

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

                        serde_json::from_str(
                            &extract_json_string(&response_text).unwrap_or_default(),
                        )?
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
                                    content: Cow::Owned(format!(
                                        "Error generating content: {}",
                                        err
                                    )),
                                })
                                .await;
                        }

                        Default::default()
                    }
                }
            }
            #[cfg(feature = "xai")]
            ClientType::Xai(xai_client) => {
                let messages = vec![XaiMessage {
                    role: "user".into(),
                    content: request.to_string(),
                }];
                let rb = ChatCompletionsRequestBuilder::new(
                    xai_client.clone(),
                    "grok-beta".into(),
                    messages,
                )
                .temperature(0.0)
                .stream(false);

                let req = rb.clone().build()?;
                let resp = rb.create_chat_completion(req).await;

                match resp {
                    Ok(chat) => {
                        let text = &chat.choices[0].message.content;
                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(text.clone()),
                        });
                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(text.clone()),
                                })
                                .await;
                        }
                        serde_json::from_str(&extract_json_string(text).unwrap_or_default())?
                    }
                    Err(err) => {
                        let msg = format!("Error generating content: {}", err);
                        error!("[*] {:?}: {:?}", self.agent.position(), err);
                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(msg.clone()),
                        });
                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(msg),
                                })
                                .await;
                        }
                        return Err(anyhow::anyhow!("XAI failure"));
                    }
                }
            }
            #[allow(unreachable_patterns)]
            _ => {
                return Err(anyhow::anyhow!(
                    "No valid AI client configured. Enable `gem`, `oai`, `cld`, or `xai` feature."
                ));
            }
        };

        tasks.scope = Some(response);
        self.agent.update(Status::Completed);
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(response)
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
    #[allow(unreachable_code)]
    #[allow(unused)]
    pub async fn get_urls(&mut self, tasks: &mut Task) -> Result<()> {
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

        let response: Vec<Cow<'static, str>> = match &mut self.client {
            #[cfg(feature = "gem")]
            ClientType::Gemini(gem_client) => {
                let parameters = ChatBuilder::default()
                    .messages(vec![Message::User {
                        content: Content::Text(request),
                        name: None,
                    }])
                    .build()?;

                let result = gem_client.chat().generate(parameters).await;

                match result {
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
                }
            }

            #[cfg(feature = "oai")]
            ClientType::OpenAI(oai_client) => {
                let parameters = ChatCompletionParametersBuilder::default()
                    .model(FlagshipModel::Gpt4O.to_string())
                    .messages(vec![ChatMessage::User {
                        content: ChatMessageContent::Text(request),
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

                        debug!(
                            "[*] {:?}: Got Response {:?}",
                            self.agent.position(),
                            response_text
                        );

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

                        serde_json::from_str(&extract_array(&response_text).unwrap_or_default())?
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
                }
            }
            #[cfg(feature = "cld")]
            ClientType::Anthropic(client) => {
                let body = CreateMessageParams::new(RequiredMessageParams {
                    model: "claude-3-7-sonnet-latest".to_string(),
                    messages: vec![AnthMessage::new_text(Role::User, request.clone())],
                    max_tokens: 1024,
                });

                match client.create_message(Some(&body)).await {
                    Ok(chat_response) => {
                        let response_text = chat_response
                            .content
                            .iter()
                            .map(|block| match block {
                                ContentBlock::Text { text, .. } => text.as_str(),
                                _ => "",
                            })
                            .collect::<Vec<_>>()
                            .join("\n");

                        debug!(
                            "[*] {:?}: Got Response {:?}",
                            self.agent.position(),
                            response_text
                        );

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

                        serde_json::from_str(&extract_array(&response_text).unwrap_or_default())?
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
                }
            }
            #[cfg(feature = "xai")]
            ClientType::Xai(xai_client) => {
                let messages = vec![XaiMessage {
                    role: "user".into(),
                    content: request.to_string(),
                }];
                let rb = ChatCompletionsRequestBuilder::new(
                    xai_client.clone(),
                    "grok-beta".into(),
                    messages,
                )
                .temperature(0.0)
                .stream(false);

                let req = rb.clone().build()?;
                let resp = rb.create_chat_completion(req).await;

                match resp {
                    Ok(chat) => {
                        let text = &chat.choices[0].message.content;
                        debug!("[*] {:?}: Got Response {:?}", self.agent.position(), text);
                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(text.clone()),
                        });
                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(text.clone()),
                                })
                                .await;
                        }
                        serde_json::from_str(&extract_array(text).unwrap_or_default())?
                    }
                    Err(err) => {
                        let msg = format!("Error getting URLs: {}", err);
                        error!("[*] {:?}: {:?}", self.agent.position(), err);
                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(msg.clone()),
                        });
                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(msg),
                                })
                                .await;
                        }
                        return Err(anyhow::anyhow!("XAI failure"));
                    }
                }
            }

            #[allow(unreachable_patterns)]
            _ => {
                return Err(anyhow::anyhow!(
                    "No valid AI client configured. Enable `gem`, `oai`, `cld`, or `xai` feature."
                ));
            }
        };

        tasks.urls = Some(response);
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
    #[allow(unreachable_code)]
    #[allow(unused)]
    pub async fn generate_diagram(&mut self, tasks: &mut Task) -> Result<String> {
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

        let response_text: String = match &mut self.client {
            #[cfg(feature = "gem")]
            ClientType::Gemini(gem_client) => {
                let parameters = ChatBuilder::default()
                    .messages(vec![Message::User {
                        content: Content::Text(request),
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
                        let error_msg = format!("Error generating diagram: {}", err);

                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(error_msg.clone()),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(error_msg),
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
                        content: ChatMessageContent::Text(request),
                        name: None,
                    }])
                    .response_format(ChatCompletionResponseFormat::Text)
                    .build()?;

                let result = oai_client.chat().create(parameters).await;

                match result {
                    Ok(chat_response) => {
                        let message = &chat_response.choices[0].message;

                        let response = match message {
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
                        let error_msg = format!("Error generating diagram: {}", err);

                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(error_msg.clone()),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(error_msg),
                                })
                                .await;
                        }

                        Default::default()
                    }
                }
            }

            #[cfg(feature = "cld")]
            ClientType::Anthropic(client) => {
                use anthropic_ai_sdk::types::message::{
                    ContentBlock, CreateMessageParams, Message as AnthMessage,
                    RequiredMessageParams, Role,
                };

                let body = CreateMessageParams::new(RequiredMessageParams {
                    model: "claude-3-7-sonnet-latest".to_string(),
                    messages: vec![AnthMessage::new_text(Role::User, request.clone())],
                    max_tokens: 1024,
                });

                match client.create_message(Some(&body)).await {
                    Ok(chat_response) => {
                        let response = chat_response
                            .content
                            .iter()
                            .map(|block| match block {
                                ContentBlock::Text { text, .. } => text.as_str(),
                                _ => "",
                            })
                            .collect::<Vec<_>>()
                            .join("\n");

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
                        let error_msg = format!("Error generating diagram: {}", err);

                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(error_msg.clone()),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(error_msg),
                                })
                                .await;
                        }

                        Default::default()
                    }
                }
            }

            #[cfg(feature = "xai")]
            ClientType::Xai(xai_client) => {
                let messages = vec![XaiMessage {
                    role: "user".into(),
                    content: request.to_string(),
                }];
                let rb = ChatCompletionsRequestBuilder::new(
                    xai_client.clone(),
                    "grok-beta".into(),
                    messages,
                )
                .temperature(0.0)
                .stream(false);

                let req = rb.clone().build()?;
                let resp = rb.create_chat_completion(req).await;

                match resp {
                    Ok(chat) => {
                        let response = chat.choices[0].message.content.clone();
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
                        let msg = format!("Error generating diagram: {}", err);
                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(msg.clone()),
                        });
                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(msg),
                                })
                                .await;
                        }
                        return Err(anyhow::anyhow!("XAI failure"));
                    }
                }
            }

            #[allow(unreachable_patterns)]
            _ => {
                return Err(anyhow::anyhow!(
                    "No valid AI client configured. Enable `gem`, `oai`, `cld`, or `xai` feature."
                ));
            }
        };

        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(response_text)
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

/// Implementation of the trait `AgentExecutor` for `ArchitectGPT`.
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
#[async_trait]
impl AgentExecutor for ArchitectGPT {
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
    async fn execute<'a>(
        &'a mut self,
        tasks: &'a mut Task,
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
                    match fs::write(path, python_code.clone()).await {
                        Ok(_) => debug!("File 'diagram.py' created successfully!"),
                        Err(e) => error!("Error creating file 'diagram.py': {}", e),
                    }

                    for attempt in 1..=max_tries {
                        let run_python = Command::new("sh")
                            .arg("-c")
                            .arg(format!("timeout {} '.venv/bin/python' ./diagram.py", 10))
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

                                    match fs::write(path, python_code.clone()).await {
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
}
