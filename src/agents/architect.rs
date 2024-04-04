use crate::agents::agent::AgentGPT;
use crate::common::utils::{extract_array, extract_json_string};
use crate::common::utils::{strip_code_blocks, Scope, Status, Tasks};
use crate::prompts::architect::{
    ARCHITECT_DIAGRAM_PROMPT, ARCHITECT_ENDPOINTS_PROMPT, ARCHITECT_SCOPE_PROMPT,
};
use crate::traits::agent::Agent;
use crate::traits::functions::Functions;
use anyhow::Result;
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

#[derive(Debug, Clone)]
pub struct ArchitectGPT {
    agent: AgentGPT,
    client: Client,
    req_client: ReqClient,
}

impl ArchitectGPT {
    pub fn new(objective: &'static str, position: &'static str) -> Self {
        let architect_path = var("ARCHITECT_WORKSPACE")
            .unwrap_or("workspace/architect".to_string())
            .to_owned();

        if !Path::new(&architect_path).exists() {
            match fs::create_dir_all(architect_path.clone()) {
                Ok(_) => debug!("Directory '{}' created successfully!", architect_path),
                Err(e) => error!("Error creating directory '{}': {}", architect_path, e),
            }
        } else {
            debug!("Directory '{}' already exists.", architect_path);
        }

        match fs::write(architect_path.clone() + "/diagram.py", "") {
            Ok(_) => debug!("File 'diagram.py' created successfully!"),
            Err(e) => error!("Error creating file 'diagram.py': {}", e),
        }

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
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

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
                    debug!(
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
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(())
    }

    pub async fn generate_diagram(&mut self, tasks: &mut Tasks) -> Result<String> {
        let request: String = format!(
            "{}\n\nUser Request:{}",
            ARCHITECT_DIAGRAM_PROMPT, tasks.description
        );

        let gemini_response: String = match self.client.generate_content(&request).await {
            Ok(response) => strip_code_blocks(&response),
            Err(_err) => Default::default(),
        };

        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }

    pub fn agent(&self) -> &AgentGPT {
        &self.agent
    }
}

impl Functions for ArchitectGPT {
    fn get_agent(&self) -> &AgentGPT {
        &self.agent
    }

    async fn execute(&mut self, tasks: &mut Tasks, _execute: bool, max_tries: u64) -> Result<()> {
        info!(
            "[*] {:?}: Executing tasks: {:?}",
            self.agent.position(),
            tasks.clone()
        );

        let architect_path = var("ARCHITECT_WORKSPACE")
            .unwrap_or("workspace/architect".to_string())
            .to_owned();

        while self.agent.status() != &Status::Completed {
            match self.agent.status() {
                Status::InDiscovery => {
                    debug!("[*] {:?}: InDiscovery", self.agent.position());

                    let scope: Scope = self.get_scope(tasks).await?;

                    if scope.external {
                        let _ = self.get_urls(tasks).await;
                        self.agent.update(Status::InUnitTesting);
                    }
                }

                Status::InUnitTesting => {
                    let mut exclude: Vec<Cow<'static, str>> = Vec::new();

                    let urls: &Vec<Cow<'static, str>> = &tasks
                        .urls
                        .as_ref()
                        .map_or_else(Vec::default, |url| url.to_vec());

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

                    // Command to install diagrams using pip
                    let pip_install = Command::new("pip").arg("install").arg("diagrams").spawn();

                    match pip_install {
                        Ok(_) => debug!("Diagrams installed successfully!"),
                        Err(e) => error!("Error installing Diagrams: {}", e),
                    }

                    let python_code = self.generate_diagram(tasks).await?;

                    // Write the content to the file
                    match fs::write(architect_path.clone() + "/diagram.py", python_code.clone()) {
                        Ok(_) => debug!("File 'diagram.py' created successfully!"),
                        Err(e) => error!("Error creating file 'diagram.py': {}", e),
                    }

                    for attempt in 1..=max_tries {
                        let run_python = Command::new("timeout")
                            .arg(format!("{}s", 10))
                            .arg("python3")
                            .arg(architect_path.clone() + "/diagram.py")
                            .current_dir(architect_path.clone())
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

                                    match fs::write(
                                        architect_path.clone() + "/diagram.py",
                                        python_code.clone(),
                                    ) {
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
