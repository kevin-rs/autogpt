//! # `MailerGPT` agent.
//!
//! This module provides functionality for utilizing emails to generate text-based
//! content based on prompts using Nylas and Gemini APIs. The `MailerGPT` agent
//! understands email contents and produces textual responses tailored to user requirements.

use crate::agents::agent::AgentGPT;
use crate::common::utils::{ClientType, Communication, Status, Task};
use crate::traits::agent::Agent;
use crate::traits::functions::{AsyncFunctions, Functions};
use anyhow::Result;
use colored::*;
use nylas::client::Nylas;
use nylas::messages::Message;
use std::borrow::Cow;
use std::env::var;
use tracing::{debug, info};

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
    messages::{Content, Message as GemMessage},
    traits::CTrait,
};

#[cfg(feature = "xai")]
use x_ai::{
    chat_compl::{ChatCompletionsRequestBuilder, Message as XaiMessage},
    traits::ChatCompletionsFetcher,
};

use async_trait::async_trait;

/// Struct representing a `MailerGPT`, which manages email processing and text generation using Nylas and Gemini API.
pub struct MailerGPT {
    /// Represents the GPT agent responsible for handling email processing and text generation.
    agent: AgentGPT,
    /// Represents the Nylas client for interacting with email services.
    nylas_client: Nylas,
    /// Represents an OpenAI or Gemini client for interacting with their API.
    client: ClientType,
}

impl MailerGPT {
    /// Constructor function to create a new instance of MailerGPT.
    ///
    /// # Arguments
    ///
    /// * `objective` - Objective description for MailerGPT.
    /// * `position` - Position description for MailerGPT.
    ///
    /// # Returns
    ///
    /// (`MailerGPT`): A new instance of MailerGPT.
    ///
    /// # Business Logic
    ///
    /// - Initializes the GPT agent with the given objective and position.
    /// - Creates a Nylas client for interacting with email services.
    /// - Creates a Gemini client for interacting with Gemini API.
    ///
    pub async fn new(objective: &'static str, position: &'static str) -> Self {
        let agent: AgentGPT = AgentGPT::new_borrowed(objective, position);
        let client_id = var("NYLAS_CLIENT_ID").unwrap_or_default().to_owned();
        let client_secret = var("NYLAS_CLIENT_SECRET").unwrap_or_default().to_owned();
        let access_token = var("NYLAS_ACCESS_TOKEN").unwrap_or_default().to_owned();

        let nylas_client = Nylas::new(&client_id, &client_secret, Some(&access_token))
            .await
            .unwrap();

        let client = ClientType::from_env();

        info!(
            "{}",
            format!("[*] {:?}: 🛠️  Getting ready!", agent.position(),)
                .bright_white()
                .bold()
        );

        Self {
            agent,
            nylas_client,
            client,
        }
    }

    /// Asynchronously retrieves the latest emails.
    ///
    /// # Returns
    ///
    /// (`Result<Vec<Message>>`): Result containing a vector of messages representing the latest emails.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in retrieving emails.
    ///
    /// # Business Logic
    ///
    /// - Retrieves the latest emails using the Nylas client.
    /// - Logs the number of messages read.
    /// - Returns a subset of the last 5 emails for processing.
    ///
    pub async fn get_latest_emails(&mut self) -> Result<Vec<Message>> {
        let messages = self.nylas_client.messages().all().await.unwrap();

        info!(
            "[*] {:?}: Read {:?} Messages",
            self.agent.position(),
            messages.len()
        );

        Ok(messages[95..].to_vec())
    }
    /// Asynchronously generates text from the latest emails.
    ///
    /// # Arguments
    ///
    /// * `prompt` - A prompt for generating text based on email content.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): Result containing the generated text.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in generating text from emails.
    ///
    /// # Business Logic
    ///
    /// - Retrieves the latest emails.
    /// - Logs communications for user input and assistant response.
    /// - Constructs a request for generating text based on email content and the provided prompt.
    /// - Sends the request to the Gemini client to generate text.
    /// - Returns the generated text.
    pub async fn generate_text_from_emails(&mut self, prompt: &str) -> Result<String> {
        self.agent.add_communication(Communication {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(format!(
                "Requested to generate text based on emails with prompt: '{}'",
                prompt
            )),
        });
        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("user"),
                    content: Cow::Owned(format!(
                        "Requested to generate text based on emails with prompt: '{}'",
                        prompt
                    )),
                })
                .await;
        }
        let emails = match self.get_latest_emails().await {
            Ok(e) => e,
            Err(err) => {
                let error_msg = format!("Failed to fetch latest emails: {}", err);
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
        };

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(
                "Analyzing latest emails and generating text based on provided prompt..."
                    .to_string(),
            ),
        });
        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(
                        "Analyzing latest emails and generating text based on provided prompt..."
                            .to_string(),
                    ),
                })
                .await;
        }

        let gemini_response = match &mut self.client {
            #[cfg(feature = "gem")]
            ClientType::Gemini(gem_client) => {
                let parameters = ChatBuilder::default()
                    .messages(vec![GemMessage::User {
                        content: Content::Text(format!(
                            "User Request:{}\n\nEmails:{:?}",
                            prompt, emails
                        )),
                        name: None,
                    }])
                    .build()?;

                let result = gem_client.chat().generate(parameters).await;

                match result {
                    Ok(response) => response,
                    Err(err) => {
                        let error_msg = format!("Failed to generate content from emails: {}", err);
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
                        content: ChatMessageContent::Text(format!(
                            "User Request:{}\n\nEmails:{:?}",
                            prompt, emails
                        )),
                        name: None,
                    }])
                    .response_format(ChatCompletionResponseFormat::Text)
                    .build()?;

                let result = oai_client.chat().create(parameters).await;

                match result {
                    Ok(chat_response) => {
                        let message = &chat_response.choices[0].message;

                        match message {
                            ChatMessage::Assistant {
                                content: Some(chat_content),
                                ..
                            } => chat_content.to_string(),
                            ChatMessage::User { content, .. } => content.to_string(),
                            ChatMessage::System { content, .. } => content.to_string(),
                            ChatMessage::Developer { content, .. } => content.to_string(),
                            ChatMessage::Tool { content, .. } => content.clone(),
                            _ => String::from(""),
                        }
                    }

                    Err(err) => {
                        let error_msg = format!("Failed to generate content from emails: {}", err);
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

            #[cfg(feature = "cld")]
            ClientType::Anthropic(client) => {
                let body = CreateMessageParams::new(RequiredMessageParams {
                    model: "claude-3-7-sonnet-latest".to_string(),
                    messages: vec![AnthMessage::new_text(
                        Role::User,
                        format!("User Request:{}\n\nEmails:{:?}", prompt, emails),
                    )],
                    max_tokens: 1024,
                });

                match client.create_message(Some(&body)).await {
                    Ok(chat_response) => chat_response
                        .content
                        .iter()
                        .filter_map(|block| match block {
                            ContentBlock::Text { text, .. } => Some(text),
                            _ => None,
                        })
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("\n"),

                    Err(err) => {
                        let error_msg =
                            format!("Failed to generate content from Claude API: {}", err);
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
            #[cfg(feature = "xai")]
            ClientType::Xai(xai_client) => {
                let messages = vec![XaiMessage {
                    role: "user".into(),
                    content: format!("User Request:{}\n\nEmails:{:?}", prompt, emails),
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
                        let response_text = chat.choices[0].message.content.clone();

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

                        #[cfg(debug_assertions)]
                        debug!(
                            "[*] {:?}: Got XAI Output: {:?}",
                            self.agent.position(),
                            response_text
                        );

                        response_text
                    }

                    Err(err) => {
                        let err_msg = format!("Failed to generate content from emails: {}", err);

                        self.agent.add_communication(Communication {
                            role: Cow::Borrowed("assistant"),
                            content: Cow::Owned(err_msg.clone()),
                        });

                        #[cfg(feature = "mem")]
                        {
                            let _ = self
                                .save_ltm(Communication {
                                    role: Cow::Borrowed("assistant"),
                                    content: Cow::Owned(err_msg.clone()),
                                })
                                .await;
                        }

                        return Err(anyhow::anyhow!(err_msg));
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

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(
                "Generated text from emails based on the given prompt.".to_string(),
            ),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(
                        "Generated text from emails based on the given prompt.".to_string(),
                    ),
                })
                .await;
        }

        info!(
            "[*] {:?}: Got Response: {:?}",
            self.agent.position(),
            gemini_response
        );

        Ok(gemini_response)
    }
}

impl Functions for MailerGPT {
    /// Retrieves a reference to the agent.
    ///
    /// # Returns
    ///
    /// (`&AgentGPT`): A reference to the agent.
    ///
    fn get_agent(&self) -> &AgentGPT {
        &self.agent
    }
}

/// Implementation of the trait `AsyncFunctions` for MailerGPT.
/// Contains additional methods related to email processing and text generation.
///
/// This trait provides methods for:
///
/// - Retrieving the GPT agent associated with MailerGPT.
/// - Executing email processing and text generation tasks asynchronously.
///
/// # Business Logic
///
/// - Provides access to the GPT agent associated with the MailerGPT instance.
/// - Executes email processing and text generation tasks asynchronously based on the current status of the agent.
/// - Handles task execution including email retrieval and text generation.
/// - Manages retries and error handling during task execution.
#[async_trait]
impl AsyncFunctions for MailerGPT {
    /// Asynchronously executes email processing and text generation tasks associated with MailerGPT.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to tasks to be executed.
    /// * `execute` - A boolean indicating whether to execute the tasks (TODO).
    /// * `max_tries` - Maximum number of attempts to execute tasks (TODO).
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
    /// - Executes email processing and text generation tasks asynchronously based on the current status of the agent.
    /// - Handles task execution including email retrieval and text generation.
    /// - Manages retries and error handling during task execution.
    ///
    async fn execute<'a>(
        &'a mut self,
        tasks: &'a mut Task,
        _execute: bool,
        _browse: bool,
        _max_tries: u64,
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
        let mut _count = 0;
        while self.agent.status() != &Status::Completed {
            match self.agent.status() {
                Status::Idle => {
                    debug!("[*] {:?}: Idle", self.agent.position());

                    let _generated_text =
                        self.generate_text_from_emails(&tasks.description).await?;

                    _count += 1;
                    self.agent.update(Status::Completed);
                }
                _ => {
                    self.agent.update(Status::Completed);
                }
            }
        }

        Ok(())
    }

    #[cfg(any(feature = "oai", feature = "gem", feature = "cld"))]
    async fn send_request(&mut self, _request: &str) -> Result<String> {
        Ok("".to_string())
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
