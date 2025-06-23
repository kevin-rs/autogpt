#![doc = include_str!("../INSTALLATION.md")]

use crate::agents::agent::AgentGPT;
pub use crate::common::utils::{ClientType, Message, Model, Tool};

use derive_builder::Builder;
#[cfg(any(feature = "oai", feature = "gem", feature = "cld"))]
use {
    crate::agents::architect::ArchitectGPT,
    crate::agents::backend::BackendGPT,
    crate::agents::frontend::FrontendGPT,
    crate::agents::manager::ManagerGPT,
    crate::agents::optimizer::OptimizerGPT,
    crate::common::utils::{Communication, Scope, Status, Tasks, strip_code_blocks},
    crate::traits::agent::Agent,
    crate::traits::functions::Functions,
    anyhow::{Result, anyhow},
    tracing::{debug, error},
};

#[cfg(feature = "img")]
use crate::agents::designer::DesignerGPT;
#[cfg(feature = "git")]
use crate::agents::git::GitGPT;
#[cfg(feature = "mail")]
use crate::agents::mailer::MailerGPT;

pub use std::borrow::Cow;
pub use uuid::Uuid;

#[cfg(feature = "mem")]
use {
    crate::common::memory::load_long_term_memory, crate::common::memory::long_term_memory_context,
    crate::common::memory::save_long_term_memory,
};

#[cfg(feature = "oai")]
use {openai_dive::v1::models::FlagshipModel, openai_dive::v1::resources::chat::*};

#[cfg(feature = "cld")]
use anthropic_ai_sdk::types::message::{
    ContentBlock, CreateMessageParams, Message as AnthMessage, MessageClient, MessageContent,
    RequiredMessageParams, Role,
};

#[cfg(feature = "gem")]
use gems::{
    chat::ChatBuilder,
    messages::{Content, Message as GeminiMessage},
    traits::CTrait,
};

#[cfg(feature = "xai")]
use x_ai::{
    chat_compl::{ChatCompletionsRequestBuilder, Message as XaiMessage},
    traits::ChatCompletionsFetcher,
};

#[derive(Builder, Default, Clone)]
#[allow(unreachable_code)]
#[builder(setter(into, strip_option), default)]
pub struct AutoGPT {
    /// Unique identifier for the agent.
    pub id: Uuid,
    /// Represents a provider for interacting with an AI API (OpenAI, Gemini, Claude, or XAI).
    pub provider: ClientType,
    /// Represents AI tools to be used by the AI provider.
    pub tools: Vec<Tool>,
    /// Represents the GPT agent responsible for handling tasks.
    pub agent: AgentGPT,
}

impl AutoGPT {
    #[cfg(any(feature = "oai", feature = "gem", feature = "cld"))]
    pub async fn run(&mut self, messages: Vec<Message>) -> Result<String> {
        let description = match messages.first() {
            #[cfg(feature = "oai")]
            Some(Message::OpenAI(ChatMessage::User {
                content: ChatMessageContent::Text(text),
                ..
            })) => text.clone(),

            #[cfg(feature = "gem")]
            Some(Message::Gemini(GeminiMessage::User {
                content: Content::Text(text),
                ..
            })) => text.clone(),

            #[cfg(feature = "cld")]
            Some(Message::Claude(AnthMessage {
                role: Role::User,
                content,
                ..
            })) => match content {
                MessageContent::Text { content: text } => text.clone(),
                MessageContent::Blocks { content: blocks } => blocks
                    .iter()
                    .filter_map(|block| match block {
                        ContentBlock::Text { text, .. } => Some(text.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            },

            #[cfg(feature = "xai")]
            Some(Message::Xai(XaiMessage { role, content })) if role == "user" => content.clone(),

            _ => "No task description provided.".to_string(),
        };

        let mut tasks = Tasks {
            description: description.into(),
            scope: Some(Scope {
                crud: true,
                auth: false,
                external: true,
            }),
            urls: None,
            frontend_code: None,
            backend_code: None,
            api_schema: None,
        };

        if self.tools.is_empty() {
            debug!("No tools specified; using raw model request via a client provider");

            let request = tasks.description;

            self.agent.add_communication(Communication {
                role: Cow::Borrowed("user"),
                content: request.clone(),
            });

            #[cfg(feature = "mem")]
            {
                let _ = self
                    .save_ltm(Communication {
                        role: Cow::Borrowed("user"),
                        content: request.clone(),
                    })
                    .await;
            }

            let response: String = match &self.provider {
                #[cfg(feature = "gem")]
                ClientType::Gemini(gem_client) => {
                    let parameters = ChatBuilder::default()
                        .messages(vec![GeminiMessage::User {
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
                            self.agent.add_communication(Communication {
                                role: Cow::Borrowed("assistant"),
                                content: Cow::Owned(format!("Error generating: {}", err)),
                            });

                            #[cfg(feature = "mem")]
                            {
                                let _ = self
                                    .save_ltm(Communication {
                                        role: Cow::Borrowed("assistant"),
                                        content: Cow::Owned(format!("Error generating: {}", err)),
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
                            self.agent.add_communication(Communication {
                                role: Cow::Borrowed("assistant"),
                                content: Cow::Owned(format!(
                                    "Error generating backend code: {}",
                                    err
                                )),
                            });

                            #[cfg(feature = "mem")]
                            {
                                let _ = self
                                    .save_ltm(Communication {
                                        role: Cow::Borrowed("assistant"),
                                        content: Cow::Owned(format!(
                                            "Error generating backend code: {}",
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
                        messages: vec![AnthMessage::new_text(Role::User, request.to_string())],
                        max_tokens: 1024,
                    });

                    match client.create_message(Some(&body)).await {
                        Ok(chat_response) => {
                            let response_text = chat_response
                                .content
                                .iter()
                                .filter_map(|block| match block {
                                    ContentBlock::Text { text, .. } => Some(text),
                                    _ => None,
                                })
                                .cloned()
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

                            strip_code_blocks(&response_text)
                        }

                        Err(err) => {
                            let error_message = format!("Error generating backend code: {}", err);

                            self.agent.add_communication(Communication {
                                role: Cow::Borrowed("assistant"),
                                content: Cow::Owned(error_message.clone()),
                            });

                            #[cfg(feature = "mem")]
                            {
                                let _ = self
                                    .save_ltm(Communication {
                                        role: Cow::Borrowed("assistant"),
                                        content: Cow::Owned(error_message.clone()),
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
                        role: "user".to_string(),
                        content: request.to_string(),
                    }];

                    let request_builder = ChatCompletionsRequestBuilder::new(
                        xai_client.clone(),
                        "grok-beta".to_string(),
                        messages,
                    )
                    .temperature(0.0)
                    .stream(false);

                    let request = match request_builder.clone().build() {
                        Ok(req) => req,
                        Err(err) => {
                            let error_message = format!("Failed to build Xai request: {}", err);
                            self.agent.add_communication(Communication {
                                role: Cow::Borrowed("assistant"),
                                content: Cow::Owned(error_message.clone()),
                            });
                            #[cfg(feature = "mem")]
                            {
                                let _ = self
                                    .save_ltm(Communication {
                                        role: Cow::Borrowed("assistant"),
                                        content: Cow::Owned(error_message.clone()),
                                    })
                                    .await;
                            }
                            return Err(anyhow!(error_message));
                        }
                    };

                    let response = request_builder.create_chat_completion(request).await;
                    match response {
                        Ok(chat_response) => {
                            let response_text = chat_response.choices[0].message.content.clone();

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
                            let error_text = format!("Error generating backend code: {}", err);
                            self.agent.add_communication(Communication {
                                role: Cow::Borrowed("assistant"),
                                content: Cow::Owned(error_text.clone()),
                            });

                            #[cfg(feature = "mem")]
                            {
                                let _ = self
                                    .save_ltm(Communication {
                                        role: Cow::Borrowed("assistant"),
                                        content: Cow::Owned(error_text.clone()),
                                    })
                                    .await;
                            }

                            Default::default()
                        }
                    }
                }

                #[allow(unreachable_patterns)]
                _ => {
                    return Err(anyhow!(
                        "No valid AI client configured. Enable `gem` or `oai` feature."
                    ));
                }
            };
            self.agent.update(Status::Completed);
            return Ok(response);
        }

        for tool in &self.tools {
            match tool {
                Tool::Diagram => {
                    debug!("Tool: Diagram -> Using ArchitectGPT");
                    let mut agent =
                        ArchitectGPT::new("Design intelligent diagram-based systems", "Architect");
                    agent.execute(&mut tasks, true, false, 3).await?;
                    self.agent = agent.get_agent().clone();
                }

                Tool::Backend => {
                    debug!("Tool: Backend -> Using BackendGPT");
                    let mut agent = BackendGPT::new(
                        "Develop high-performance calculation logic",
                        "Backend",
                        "rust",
                    );
                    agent.execute(&mut tasks, true, false, 3).await?;
                    self.agent = agent.get_agent().clone();
                }

                #[cfg(feature = "img")]
                Tool::ImgGen => {
                    debug!("Tool: ImgGen -> Using DesignerGPT");
                    let mut agent = DesignerGPT::new("Design with visuals", "Designer");
                    agent.execute(&mut tasks, true, false, 3).await?;
                    self.agent = agent.get_agent().clone();
                }

                #[cfg(feature = "git")]
                Tool::Git => {
                    debug!("Tool: Git -> Using GitGPT");
                    let mut agent = GitGPT::new("Manage version control tasks", "GitGPT");
                    agent.execute(&mut tasks, true, false, 1).await?;
                    self.agent = agent.get_agent().clone();
                }

                #[cfg(feature = "mail")]
                Tool::Email => {
                    debug!("Tool: Email -> Using MailerGPT");
                    let mut agent = MailerGPT::new("Summarize and compose emails", "Mailer").await;
                    agent.execute(&mut tasks, true, false, 3).await?;
                    self.agent = agent.get_agent().clone();
                }

                Tool::Plan => {
                    debug!("Tool: Plan -> Using ManagerGPT");
                    let mut agent = ManagerGPT::new(
                        "Manage software project plans",
                        "Manager",
                        &tasks.description,
                        "python",
                    );
                    agent.execute(true, false, 3).await?;
                }

                Tool::Optimize => {
                    debug!("Tool: Optimize -> Using OptimizerGPT");
                    let mut agent = OptimizerGPT::new("Optimize source code", "Optimizer", "rust");
                    agent.execute(&mut tasks, true, false, 3).await?;
                    self.agent = agent.get_agent().clone();
                }

                Tool::Frontend => {
                    debug!("Tool: Frontend -> Using FrontendGPT");
                    let mut agent =
                        FrontendGPT::new("Develop high-performance UI", "frontend", "rust");
                    agent.execute(&mut tasks, true, false, 3).await?;
                    self.agent = agent.get_agent().clone();
                }

                _ => {
                    error!("Unsupported tool: {:?}", tool);
                    return Err(anyhow::anyhow!("Unsupported tool: {:?}", tool));
                }
            }
        }

        Ok(format!("Execution completed with tools: {:?}", self.tools))
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
        save_long_term_memory(&mut self.provider, self.agent.id.clone(), communication).await
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
    #[allow(unused)]
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
    #[allow(unused)]
    async fn ltm_context(&self) -> String {
        long_term_memory_context(self.agent.id.clone()).await
    }
}
