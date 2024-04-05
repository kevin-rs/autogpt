//! # `MailerGPT` agent.
//!
//! This module provides functionality for utilizing emails to generate text-based
//! content based on prompts using Nylas and Gemini APIs. The `MailerGPT` agent
//! understands email contents and produces textual responses tailored to user requirements.

use crate::agents::agent::AgentGPT;
use crate::common::utils::{Status, Tasks};
use crate::traits::agent::Agent;
use crate::traits::functions::Functions;
use anyhow::Result;
use gems::Client;
use nylas::client::Nylas;
use nylas::messages::Message;
use std::env::var;
use tracing::{debug, info};

/// Struct representing a `MailerGPT`, which manages email processing and text generation using Nylas and Gemini API.
pub struct MailerGPT {
    /// Represents the GPT agent responsible for handling email processing and text generation.
    agent: AgentGPT,
    /// Represents the Nylas client for interacting with email services.
    nylas_client: Nylas,
    /// Represents the Gemini client for interacting with Gemini API.
    client: Client,
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

        let model = var("GEMINI_MODEL")
            .unwrap_or("gemini-pro".to_string())
            .to_owned();
        let api_key = var("GEMINI_API_KEY").unwrap_or_default().to_owned();
        let client = Client::new(&api_key, &model);

        info!("[*] {:?}: {:?}", position, agent);

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
    /// - Constructs a request for generating text based on email content and the provided prompt.
    /// - Sends the request to the Gemini client to generate text.
    /// - Returns the generated text.
    ///
    pub async fn generate_text_from_emails(&mut self, prompt: &str) -> Result<String> {
        let emails = self.get_latest_emails().await.unwrap();

        // TODO: Parse emails bodies cz Gemini ain't geminiin'
        let gemini_response: String = match self
            .client
            .generate_content(&format!("User Request:{}\n\nEmails:{:?}", prompt, emails))
            .await
        {
            Ok(response) => response,
            Err(_err) => Default::default(),
        };

        info!(
            "[*] {:?}: Got Response: {:?}",
            self.agent.position(),
            gemini_response
        );

        Ok(gemini_response)
    }
}

/// Implementation of the trait Functions for MailerGPT.
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
    async fn execute(&mut self, tasks: &mut Tasks, _execute: bool, _max_tries: u64) -> Result<()> {
        info!(
            "[*] {:?}: Executing tasks: {:?}",
            self.agent.position(),
            tasks.clone()
        );
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
}
