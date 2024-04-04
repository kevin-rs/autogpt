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

pub struct MailerGPT {
    agent: AgentGPT,
    nylas_client: Nylas,
    client: Client,
}

impl MailerGPT {
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

    pub async fn get_latest_emails(&mut self) -> Result<Vec<Message>> {
        let messages = self.nylas_client.messages().all().await.unwrap();

        info!(
            "[*] {:?}: Read {:?} Messages",
            self.agent.position(),
            messages.len()
        );

        Ok(messages[95..].to_vec())
    }

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

impl Functions for MailerGPT {
    fn get_agent(&self) -> &AgentGPT {
        &self.agent
    }

    async fn execute(&mut self, tasks: &mut Tasks, _execute: bool, _max_tries: u64) -> Result<()> {
        info!(
            "[*] {:?}: Executing tasks: {:?}",
            self.agent.position(),
            tasks.clone()
        );
        let mut _count = 0;
        while self.agent.status() != &Status::Completed {
            match self.agent.status() {
                Status::InDiscovery => {
                    debug!("[*] {:?}: InDiscovery", self.agent.position());

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
