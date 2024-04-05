use crate::agents::agent::AgentGPT;
use crate::common::utils::{strip_code_blocks, Status, Tasks};
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
use std::io::Read;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct FrontendGPT {
    workspace: Cow<'static, str>,
    agent: AgentGPT,
    client: Client,
    req_client: ReqClient,
    bugs: Option<Cow<'static, str>>,
    language: &'static str,
    nb_bugs: u64,
}

impl FrontendGPT {
    pub fn new(objective: &'static str, position: &'static str, language: &'static str) -> Self {
        let workspace = var("AUTOGPT_WORKSPACE")
            .unwrap_or("workspace/".to_string())
            .to_owned()
            + "frontend";

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

        let cargo_new = Command::new("cargo")
            .arg("new")
            .arg(workspace.clone())
            .spawn();

        match cargo_new {
            Ok(_) => debug!("Cargo project initialized successfully!"),
            Err(e) => error!("Error initializing Cargo project: {}", e),
        }

        match fs::write(workspace.clone() + "/main.py", "") {
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
            workspace: workspace.into(),
            agent,
            client,
            req_client,
            bugs: None,
            language: language,
            nb_bugs: 0,
        }
    }

    pub async fn generate_frontend_code(&mut self, tasks: &mut Tasks) -> Result<String> {
        let path = &self.workspace;

        let full_path = match self.language {
            "rust" => {
                format!("{}/{}", path, "src/template.rs")
            }
            "python" => {
                format!("{}/{}", path, "template.py")
            }
            "javascript" => {
                format!("{}/{}", path, "src/template.js")
            }
            _ => panic!("Unsupported language, consider open an Issue/PR"),
        };

        debug!("[*] {:?}: {:?}", self.agent.position(), full_path);

        let template = fs::read_to_string(full_path)?;

        let request: String = format!(
            "{}\n\nCode Template: {}\nProject Description: {}",
            FRONTEND_CODE_PROMPT, template, tasks.description
        );

        let gemini_response: String = match self.client.generate_content(&request).await {
            Ok(response) => strip_code_blocks(&response),
            Err(_err) => Default::default(),
        };

        let frontend_path = match self.language {
            "rust" => {
                format!("{}/{}", path, "src/main.rs")
            }
            "python" => {
                format!("{}/{}", path, "main.py")
            }
            "javascript" => {
                format!("{}/{}", path, "src/index.js")
            }
            _ => panic!("Unsupported language, consider open an Issue/PR"),
        };

        fs::write(frontend_path, gemini_response.clone())?;

        tasks.frontend_code = Some(gemini_response.clone().into());

        self.agent.update(Status::Completed);
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }

    pub async fn improve_frontend_code(&mut self, tasks: &mut Tasks) -> Result<String> {
        let path = &self.workspace;

        let request: String = format!(
            "{}\n\nCode Template: {}\nProject Description: {}",
            IMPROVED_FRONTEND_CODE_PROMPT,
            tasks.clone().frontend_code.unwrap_or_default(),
            tasks.description
        );

        let gemini_response: String = match self.client.generate_content(&request).await {
            Ok(response) => strip_code_blocks(&response),
            Err(_err) => Default::default(),
        };

        let frontend_path = match self.language {
            "rust" => {
                format!("{}/{}", path, "src/main.rs")
            }
            "python" => {
                format!("{}/{}", path, "main.py")
            }
            "javascript" => {
                format!("{}/{}", path, "src/index.js")
            }
            _ => panic!("Unsupported language, consider open an Issue/PR"),
        };

        debug!("[*] {:?}: {:?}", self.agent.position(), frontend_path);

        fs::write(frontend_path, gemini_response.clone())?;

        tasks.frontend_code = Some(gemini_response.clone().into());

        self.agent.update(Status::Completed);
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }

    pub async fn fix_code_bugs(&mut self, tasks: &mut Tasks) -> Result<String> {
        let path = &self.workspace;

        let request: String = format!(
            "{}\n\nBuggy Code: {}\nBugs: {}\n\nFix all bugs.",
            FIX_CODE_PROMPT,
            tasks.clone().frontend_code.unwrap(),
            self.bugs.clone().unwrap()
        );

        let gemini_response: String = match self.client.generate_content(&request).await {
            Ok(response) => strip_code_blocks(&response),
            Err(_err) => Default::default(),
        };

        let frontend_path = match self.language {
            "rust" => {
                format!("{}/{}", path, "src/main.rs")
            }
            "python" => {
                format!("{}/{}", path, "main.py")
            }
            "javascript" => {
                format!("{}/{}", path, "src/index.js")
            }
            _ => panic!("Unsupported language, consider open an Issue/PR"),
        };

        fs::write(frontend_path, gemini_response.clone())?;

        tasks.frontend_code = Some(gemini_response.clone().into());

        self.agent.update(Status::Completed);
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

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

    async fn execute(&mut self, tasks: &mut Tasks, execute: bool, max_tries: u64) -> Result<()> {
        info!(
            "[*] {:?}: Executing tasks: {:?}",
            self.agent.position(),
            tasks.clone()
        );

        let path = &self.workspace.to_string();

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
                    }

                    info!(
                "[*] {:?}: Frontend Code Unit Testing: Building and running the frontend project...",
                self.agent.position(),
            );

                    let result = match self.language {
                        "rust" => {
                            let mut build_command = Command::new("timeout");
                            build_command
                                .arg(format!("{}s", 10))
                                .arg("cargo")
                                .arg("build")
                                .arg("--release")
                                .current_dir(&path.to_string())
                                .stdout(Stdio::piped())
                                .stderr(Stdio::piped())
                                .spawn()
                        }
                        "python" => Command::new("timeout")
                            .arg(format!("{}s", 10))
                            .arg("uvicorn")
                            .arg("main:app")
                            .current_dir(&path.to_string())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn(),
                        "javascript" => Command::new("timeout")
                            .arg(format!("{}s", 10))
                            .arg("npm")
                            .arg("run")
                            .arg("build")
                            .current_dir(&path.to_string())
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
                                info!(
                                        "[*] {:?}: Frontend Code Unit Testing: Too many bugs found in the code. Consider debugging...",
                                        self.agent.position(),
                                    );
                                break;
                            } else {
                                self.agent.update(Status::Active);
                            }
                            if !stderr_output.trim().is_empty() {
                                self.bugs = Some(stderr_output.into());
                            } else {
                                info!(
                                    "[*] {:?}: Frontend Code Unit Testing: Frontend server build successful...",
                                    self.agent.position(),
                                );
                            }
                        }
                        Err(err) => {
                            panic!("Failed to execute command: {}", err);
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
