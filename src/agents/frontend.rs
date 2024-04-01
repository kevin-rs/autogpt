use crate::agents::agent::AgentGPT;
use crate::common::utils::{Status, Tasks};
use crate::prompts::frontend::{
    FIX_CODE_PROMPT, FRONTEND_CODE_PROMPT, IMPROVED_FRONTEND_CODE_PROMPT,
};
use crate::traits::agent::Agent;
use crate::traits::functions::Functions;
use anyhow::Result;
use gems::Client;
use reqwest::Client as ReqClient;
use std::borrow::Cow;
use std::env::var;
use std::fs;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use tracing::info;

#[derive(Debug)]
#[allow(unused)]
pub struct FrontendGPT {
    agent: AgentGPT,
    client: Client,
    req_client: ReqClient,
    bugs: Option<Cow<'static, str>>,
    nb_bugs: u64,
}

impl FrontendGPT {
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
            bugs: None,
            nb_bugs: 0,
        }
    }

    pub async fn generate_frontend_code(&mut self, tasks: &mut Tasks) -> Result<String> {
        let path = var("FRONTEND_TEMPLATE_PATH")
            .unwrap_or("frontend".to_string())
            .to_owned();

        let full_path = format!("{}/{}", path, "src/template.rs");
        info!("[*] {:?}: {:?}", self.agent.position(), full_path);

        let template = fs::read_to_string(full_path)?;

        let request: String = format!(
            "{}\n\nCode Template: {}\nProject Description: {}",
            FRONTEND_CODE_PROMPT, template, tasks.description
        );

        let gemini_response: String = match self.client.generate_content(&request).await {
            Ok(response) => response,
            Err(_err) => Default::default(),
        };

        let frontend_path = format!("{}/{}", path, "src/main.rs");

        fs::write(frontend_path, gemini_response.clone())?;

        tasks.frontend_code = Some(gemini_response.clone().into());

        self.agent.update(Status::Completed);
        info!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }

    pub async fn improve_frontend_code(&mut self, tasks: &mut Tasks) -> Result<String> {
        let path = var("FRONTEND_TEMPLATE_PATH")
            .unwrap_or("frontend".to_string())
            .to_owned();

        let request: String = format!(
            "{}\n\nCode Template: {}\nProject Description: {}",
            IMPROVED_FRONTEND_CODE_PROMPT,
            tasks.clone().frontend_code.unwrap(),
            tasks.description
        );

        let gemini_response: String = match self.client.generate_content(&request).await {
            Ok(response) => response,
            Err(_err) => Default::default(),
        };

        let frontend_path = format!("{}/{}", path, "src/main.rs");
        info!("[*] {:?}: {:?}", self.agent.position(), frontend_path);

        fs::write(frontend_path, gemini_response.clone())?;

        tasks.frontend_code = Some(gemini_response.clone().into());

        self.agent.update(Status::Completed);
        info!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }

    pub async fn fix_code_bugs(&mut self, tasks: &mut Tasks) -> Result<String> {
        let path = var("FRONTEND_TEMPLATE_PATH")
            .unwrap_or("frontend".to_string())
            .to_owned();

        let request: String = format!(
            "{}\n\nBuggy Code: {}\nBugs: {}\n\nFix all bugs.",
            FIX_CODE_PROMPT,
            tasks.clone().frontend_code.unwrap(),
            self.bugs.clone().unwrap()
        );

        let gemini_response: String = match self.client.generate_content(&request).await {
            Ok(response) => response,
            Err(_err) => Default::default(),
        };

        let frontend_path = format!("{}/{}", path, "src/main.rs");

        fs::write(frontend_path, gemini_response.clone())?;

        tasks.frontend_code = Some(gemini_response.clone().into());

        self.agent.update(Status::Completed);
        info!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }

    pub fn agent(&self) -> &AgentGPT {
        &self.agent
    }

    pub fn update_bugs(&mut self, bugs: Option<Cow<'static, str>>) {
        self.bugs = bugs;
    }
}

impl Functions for FrontendGPT {
    fn get_agent(&self) -> &AgentGPT {
        &self.agent
    }

    async fn execute(&mut self, tasks: &mut Tasks, execute: bool) -> Result<()> {
        let path = var("FRONTEND_TEMPLATE_PATH")
            .unwrap_or("frontend".to_string())
            .to_owned();
        while self.agent.status() != &Status::Completed {
            match &self.agent.status() {
                Status::InDiscovery => {
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
                        "[*] {:?}: Frontend Code Unit Testing: Awaiting user confirmation for code safety...",
                        self.agent.position(),
                    );

                    if !execute {
                        info!(
                            "[*] {:?}: It seems the code isn't safe to proceed. Consider revising or seeking assistance...",
                            self.agent.position(),
                        );
                    } else {
                        info!(
                            "[*] {:?}: Frontend Code Unit Testing: Building the frontend project...",
                            self.agent.position(),
                        );

                        let serve_command = format!("trunk build --release");
                        let output = Command::new("sh")
                            .arg("-c")
                            .arg(&serve_command)
                            .current_dir(path.clone())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .output()
                            .expect("Failed to execute trunk build command");

                        if output.status.success() {
                            self.nb_bugs = 0;
                            info!(
                                "[*] {:?}: Frontend Code Unit Testing: Frontend server build successful...",
                                self.agent.position(),
                            );
                        } else {
                            let error_arr: Vec<u8> = output.stderr;
                            let error_str: String = String::from_utf8(error_arr).unwrap();

                            self.nb_bugs += 1;
                            self.bugs = Some(error_str.into());

                            if self.nb_bugs > 6 {
                                info!(
                                    "[*] {:?}: Frontend Code Unit Testing: Too many bugs found in the code. Consider debugging...",
                                    self.agent.position(),
                                );
                                break;
                            }

                            self.agent.update(Status::Active);
                            continue;
                        }
                    }
                    self.agent.update(Status::Completed);
                }
                _ => {}
            }
        }
        Ok(())
    }
}
