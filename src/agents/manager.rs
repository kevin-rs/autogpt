#![allow(unused)]

//! # `ManagerGPT` agent.
//!

use crate::agents::agent::AgentGPT;
use crate::agents::architect::ArchitectGPT;
use crate::agents::backend::BackendGPT;
#[cfg(feature = "img")]
use crate::agents::designer::DesignerGPT;
use crate::agents::frontend::FrontendGPT;
#[cfg(feature = "git")]
use crate::agents::git::GitGPT;
use crate::agents::types::AgentType;
use crate::common::utils::strip_code_blocks;
use crate::common::utils::{ClientType, Communication, Tasks};
use crate::prompts::manager::{FRAMEWORK_MANAGER_PROMPT, LANGUAGE_MANAGER_PROMPT, MANAGER_PROMPT};
use crate::traits::agent::Agent;
use crate::traits::functions::Functions;
use anyhow::Result;
use colored::*;
#[cfg(feature = "gem")]
use gems::Client;
use std::borrow::Cow;
use std::env::var;
use tracing::info;

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
    /// Represents an OpenAI or Gemini client for interacting with their API.
    client: ClientType,
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

        let client = ClientType::from_env();

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
        #[cfg(feature = "git")]
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
    pub async fn execute_prompt(&mut self, prompt: String) -> Result<String, anyhow::Error> {
        let provider = var("AI_PROVIDER").unwrap_or_else(|_| "gemini".to_string());
        let response = match &mut self.client {
            #[cfg(feature = "gem")]
            ClientType::Gemini(gem_client) if provider == "gemini" => {
                let parameters = ChatBuilder::default()
                    .messages(vec![Message::User {
                        content: Content::Text(prompt),
                        name: None,
                    }])
                    .build()?;
                let result = gem_client.chat().generate(parameters).await;

                match result {
                    Ok(response) => strip_code_blocks(&response),
                    Err(_err) => {
                        let error_msg = "Failed to generate content via Gemini API.".to_string();
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
            ClientType::OpenAI(oai_client) if provider == "openai" => {
                use openai_dive::v1::resources::chat::*;
                use openai_dive::v1::resources::model::*;

                let parameters = ChatCompletionParametersBuilder::default()
                    .model(FlagshipModel::Gpt4O.to_string())
                    .messages(vec![ChatMessage::User {
                        content: ChatMessageContent::Text(prompt.clone()),
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
                        let error_msg = "Failed to generate content via OpenAI API.".to_string();
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
            ClientType::Anthropic(client) if provider == "claude" => {
                let body = CreateMessageParams::new(RequiredMessageParams {
                    model: "claude-3-7-sonnet-latest".to_string(),
                    messages: vec![AnthMessage::new_text(Role::User, prompt.clone())],
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

                        strip_code_blocks(&response_text)
                    }

                    Err(_) => {
                        let error_msg = "Failed to generate content via Claude API.".to_string();
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

        Ok(response)
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
    pub async fn execute(&mut self, execute: bool, browse: bool, max_tries: u64) -> Result<()> {
        self.agent.add_communication(Communication {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(format!(
                "Execute tasks with description: '{}'",
                self.tasks.description.clone()
            )),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("user"),
                    content: Cow::Owned(format!(
                        "Execute tasks with description: '{}'",
                        self.tasks.description.clone()
                    )),
                })
                .await;
        }
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

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(
                        "Analyzing user request to determine programming language and framework..."
                            .to_string(),
                    ),
                })
                .await;
        }
        let language = self.execute_prompt(language_request).await?;
        let framework = self.execute_prompt(framework_request).await?;

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(format!(
                "Identified Language: '{}', Framework: '{}'",
                language, framework
            )),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(format!(
                        "Identified Language: '{}', Framework: '{}'",
                        language, framework
                    )),
                })
                .await;
        }
        if self.agents.is_empty() {
            self.spawn_default_agents();
            self.agent.add_communication(Communication {
                role: Cow::Borrowed("system"),
                content: Cow::Borrowed("No agents were available. Spawned default agents."),
            });
        }

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("system"),
                    content: Cow::Borrowed("No agents were available. Spawned default agents."),
                })
                .await;
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

            #[cfg(feature = "mem")]
            {
                let _ = self
                    .save_ltm(Communication {
                        role: Cow::Borrowed("assistant"),
                        content: Cow::Owned(format!(
                            "Refined task for '{}': {}",
                            agent.position(),
                            refined_task
                        )),
                    })
                    .await;
            }

            self.tasks = Tasks {
                description: refined_task.into(),
                scope: None,
                urls: None,
                frontend_code: None,
                backend_code: None,
                api_schema: None,
            };

            let _agent_res = agent
                .execute(&mut self.tasks, execute, browse, max_tries)
                .await;
        }

        self.agent.add_communication(Communication {
            role: Cow::Borrowed("assistant"),
            content: Cow::Borrowed("Task execution completed by all agents."),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Communication {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Borrowed("Task execution completed by all agents."),
                })
                .await;
        }
        info!(
            "{}",
            format!("[*] {:?}: Completed Tasks:", self.agent.position())
                .bright_white()
                .bold()
        );

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
