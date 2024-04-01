use crate::agents::agent::AgentGPT;
use crate::common::utils::{extract_array, extract_json_string};
use crate::common::utils::{Scope, Status, Tasks};
use crate::prompts::architect::{ARCHITECT_ENDPOINTS_PROMPT, ARCHITECT_SCOPE_PROMPT};
use crate::traits::agent::Agent;
use crate::traits::functions::Functions;
use anyhow::Result;
use gems::Client;
use reqwest::Client as ReqClient;
use std::borrow::Cow;
use std::env::var;
use std::time::Duration;
use tracing::info;

#[derive(Debug)]
pub struct ArchitectGPT {
    agent: AgentGPT,
    client: Client,
    req_client: ReqClient,
}

impl ArchitectGPT {
    pub fn new(objective: &'static str, position: &'static str) -> Self {
        let agent: AgentGPT = AgentGPT::new_borrowed(objective, position);
        let model = var("GEMINI_MODEL")
            .unwrap_or("gemini-pro".to_string())
            .to_owned();
        let api_key = var("GEMINI_API_KEY").unwrap_or_default().to_owned();
        let client = Client::new(&api_key, &model);
        info!("[*] {:?}: {:?}", position, agent);

        let req_client: ReqClient = ReqClient::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap();

        Self {
            agent,
            client,
            req_client,
        }
    }

    pub async fn get_scope(&mut self, tasks: &mut Tasks) -> Result<Scope> {
        let request: String = format!(
            "{}\n\nHere is the User Request:{}",
            ARCHITECT_SCOPE_PROMPT, tasks.description
        );

        // use prompts scope
        let gemini_response: Scope = match self.client.generate_content(&request).await {
            Ok(response) => {
                serde_json::from_str(&extract_json_string(&response).unwrap_or_default())?
            }
            Err(_err) => Default::default(),
        };

        tasks.scope = Some(gemini_response);
        self.agent.update(Status::Completed);
        info!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }

    pub async fn get_urls(&mut self, tasks: &mut Tasks) -> Result<()> {
        let request: String = format!(
            "{}\n\nHere is the Project Description:{}",
            ARCHITECT_ENDPOINTS_PROMPT, tasks.description
        );

        let gemini_response: Vec<Cow<'static, str>> =
            match self.client.generate_content(&request).await {
                Ok(response) => {
                    info!(
                        "[*] {:?}: Got Response {:?}",
                        self.agent.position(),
                        response
                    );
                    serde_json::from_str(&extract_array(&response).unwrap_or_default())?
                }
                Err(_err) => Default::default(),
            };

        tasks.urls = Some(gemini_response);
        self.agent.update(Status::InUnitTesting);
        info!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(())
    }

    pub fn agent(&self) -> &AgentGPT {
        &self.agent
    }
}

impl Functions for ArchitectGPT {
    fn get_agent(&self) -> &AgentGPT {
        &self.agent
    }

    async fn execute(&mut self, tasks: &mut Tasks, _execute: bool) -> Result<()> {
        while self.agent.status() != &Status::Completed {
            match self.agent.status() {
                Status::InDiscovery => {
                    info!("[*] {:?}: InDiscovery", self.agent.position());

                    let scope: Scope = self.get_scope(tasks).await?;

                    if scope.external {
                        let _ = self.get_urls(tasks).await;
                        self.agent.update(Status::InUnitTesting);
                    }
                }

                Status::InUnitTesting => {
                    let mut exclude: Vec<Cow<'static, str>> = Vec::new();

                    let urls: &Vec<Cow<'static, str>> = tasks.urls.as_ref().expect("No URLS found");

                    for url in urls {
                        info!(
                            "[*] {:?}: Testing URL Endpoint: {}",
                            self.agent.position(),
                            url
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
                                info!(
                                    "[*] {:?}: Error sending request for URL {}: {:?}",
                                    self.agent.position(),
                                    url,
                                    err
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
