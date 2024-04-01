use crate::agents::agent::AgentGPT;
use crate::common::utils::Route;
use crate::common::utils::{strip_code_blocks, Status, Tasks};
use crate::prompts::backend::{
    API_ENDPOINTS_PROMPT, FIX_CODE_PROMPT, IMPROVED_WEBSERVER_CODE_PROMPT, WEBSERVER_CODE_PROMPT,
};
use crate::traits::agent::Agent;
use crate::traits::functions::Functions;
use std::io::Read;
use std::process::Command;
use std::process::Stdio;
use std::thread::sleep;

use anyhow::Result;
use gems::Client;
use reqwest::Client as ReqClient;
use std::borrow::Cow;
use std::env::var;
use std::fs;
use std::time::Duration;
use tracing::info;

#[derive(Debug)]
pub struct BackendGPT {
    agent: AgentGPT,
    client: Client,
    req_client: ReqClient,
    bugs: Option<Cow<'static, str>>,
    language: &'static str,
    nb_bugs: u64,
}

impl BackendGPT {
    pub fn new(objective: &'static str, position: &'static str, language: &'static str) -> Self {
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
            language: language,
            nb_bugs: 0,
        }
    }

    pub async fn generate_backend_code(&mut self, tasks: &mut Tasks) -> Result<String> {
        let path = var("BACKEND_TEMPLATE_PATH")
            .unwrap_or("backend".to_string())
            .to_owned();

        let full_path = match self.language {
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

        info!("[*] {:?}: {:?}", self.agent.position(), full_path);

        let template = fs::read_to_string(full_path)?;

        let request: String = format!(
            "{}\n\nCode Template: {}\nProject Description: {}",
            WEBSERVER_CODE_PROMPT, template, tasks.description
        );

        let gemini_response: String = match self.client.generate_content(&request).await {
            Ok(response) => strip_code_blocks(&response),
            Err(_err) => Default::default(),
        };

        let backend_path = match self.language {
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

        fs::write(backend_path, gemini_response.clone())?;

        tasks.backend_code = Some(gemini_response.clone().into());

        self.agent.update(Status::Completed);
        info!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }

    pub async fn improve_backend_code(&mut self, tasks: &mut Tasks) -> Result<String> {
        let path = var("BACKEND_TEMPLATE_PATH")
            .unwrap_or("backend".to_string())
            .to_owned();

        let request: String = format!(
            "{}\n\nCode Template: {}\nProject Description: {}",
            IMPROVED_WEBSERVER_CODE_PROMPT,
            tasks.clone().backend_code.unwrap(),
            tasks.description
        );

        let gemini_response: String = match self.client.generate_content(&request).await {
            Ok(response) => strip_code_blocks(&response),
            Err(_err) => Default::default(),
        };

        let backend_path = match self.language {
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

        info!("[*] {:?}: {:?}", self.agent.position(), backend_path);

        fs::write(backend_path, gemini_response.clone())?;

        tasks.backend_code = Some(gemini_response.clone().into());

        self.agent.update(Status::Completed);
        info!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }

    pub async fn fix_code_bugs(&mut self, tasks: &mut Tasks) -> Result<String> {
        let path = var("BACKEND_TEMPLATE_PATH")
            .unwrap_or("backend".to_string())
            .to_owned();

        let request: String = format!(
            "{}\n\nBuggy Code: {}\nBugs: {}\n\nFix all bugs.",
            FIX_CODE_PROMPT,
            tasks.clone().backend_code.unwrap(),
            self.bugs.clone().unwrap()
        );

        let gemini_response: String = match self.client.generate_content(&request).await {
            Ok(response) => strip_code_blocks(&response),
            Err(_err) => Default::default(),
        };

        let backend_path = match self.language {
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

        fs::write(backend_path, gemini_response.clone())?;

        tasks.backend_code = Some(gemini_response.clone().into());

        self.agent.update(Status::Completed);
        info!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }

    pub async fn get_routes_json(&mut self) -> Result<String> {
        let path = var("BACKEND_TEMPLATE_PATH")
            .unwrap_or("backend".to_string())
            .to_owned();

        let full_path = match self.language {
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

        info!("[*] {:?}: {:?}", self.agent.position(), full_path);

        let backend_code = fs::read_to_string(full_path)?;

        let request: String = format!(
            "{}\n\nHere is the backend code with all routes:{}",
            API_ENDPOINTS_PROMPT, backend_code
        );

        let gemini_response: String = match self.client.generate_content(&request).await {
            Ok(response) => strip_code_blocks(&response),
            Err(_err) => Default::default(),
        };

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

impl Functions for BackendGPT {
    fn get_agent(&self) -> &AgentGPT {
        &self.agent
    }

    async fn execute(&mut self, tasks: &mut Tasks, execute: bool, max_tries: u64) -> Result<()> {
        let path = var("BACKEND_TEMPLATE_PATH")
            .unwrap_or("backend".to_string())
            .to_owned();

        while self.agent.status() != &Status::Completed {
            match &self.agent.status() {
                Status::InDiscovery => {
                    let _ = self.generate_backend_code(tasks).await;
                    self.agent.update(Status::Active);
                    continue;
                }

                Status::Active => {
                    if self.nb_bugs == 0 {
                        let _ = self.improve_backend_code(tasks).await;
                    } else {
                        let _ = self.fix_code_bugs(tasks).await;
                    }
                    self.agent.update(Status::InUnitTesting);
                    continue;
                }

                Status::InUnitTesting => {
                    info!(
                        "[*] {:?}: Backend Code Unit Testing: Awaiting user confirmation for code safety...",
                        self.agent.position(),
                    );

                    if !execute {
                        info!(
                            "[*] {:?}: It seems the code isn't safe to proceed. Consider revising or seeking assistance...",
                            self.agent.position(),
                        );
                    } else {
                        info!(
                            "[*] {:?}: Backend Code Unit Testing: Building and running the backend project...",
                            self.agent.position(),
                        );

                        let result = match self.language {
                            "rust" => {
                                let mut build_command = Command::new("cargo");
                                build_command
                                    .arg("build")
                                    .arg("--release")
                                    .arg("--verbose")
                                    .current_dir(&path);
                                let build_output = build_command
                                    .output()
                                    .expect("Failed to build the backend application");

                                if build_output.status.success() {
                                    let run_output = Command::new("timeout")
                                        .arg(format!("{}s", 10))
                                        .arg("cargo")
                                        .arg("run")
                                        .arg("--release")
                                        .arg("--verbose")
                                        .current_dir(&path)
                                        .stdout(Stdio::piped())
                                        .stderr(Stdio::piped())
                                        .spawn()
                                        .expect("Failed to run the backend application");
                                    Some(run_output)
                                } else {
                                    None
                                }
                            }
                            "python" => {
                                let run_output = Command::new("timeout")
                                    .arg(format!("{}s", 10))
                                    .arg("uvicorn")
                                    .arg("main:app")
                                    .current_dir(&path)
                                    .stdout(Stdio::piped())
                                    .stderr(Stdio::piped())
                                    .spawn()
                                    .expect("Failed to run the backend application");
                                Some(run_output)
                            }
                            "javascript" => {
                                let run_output = Command::new("timeout")
                                    .arg(format!("{}s", 10))
                                    .arg("node")
                                    .arg("server.js")
                                    .current_dir(&path)
                                    .stdout(Stdio::piped())
                                    .stderr(Stdio::piped())
                                    .spawn()
                                    .expect("Failed to run the backend application");
                                Some(run_output)
                            }
                            _ => None,
                        };

                        if let Some(mut child) = result {
                            let _build_stdout =
                                child.stdout.take().expect("Failed to capture build stdout");
                            let mut build_stderr =
                                child.stderr.take().expect("Failed to capture build stderr");

                            let mut stderr_output = String::new();
                            build_stderr.read_to_string(&mut stderr_output).unwrap();

                            if !stderr_output.trim().is_empty() {
                                self.nb_bugs += 1;
                                self.bugs = Some(stderr_output.into());

                                if self.nb_bugs > max_tries {
                                    info!(
                                        "[*] {:?}: Backend Code Unit Testing: Too many bugs found in the code. Consider debugging...",
                                        self.agent.position(),
                                    );
                                    break;
                                }

                                self.agent.update(Status::Active);
                                continue;
                            } else {
                                self.nb_bugs = 0;
                                info!(
                                    "[*] {:?}: Backend Code Unit Testing: Backend server build successful...",
                                    self.agent.position(),
                                );
                            }

                            let seconds_sleep = Duration::from_secs(3);
                            sleep(seconds_sleep);

                            let endpoints: String = self.get_routes_json().await?;

                            let api_endpoints: Vec<Route> =
                                serde_json::from_str(endpoints.as_str())
                                    .expect("Failed to decode API Endpoints");

                            let filtered_endpoints: Vec<Route> = api_endpoints
                                .iter()
                                .filter(|&route| route.method == "get" && route.dynamic == "false")
                                .cloned()
                                .collect();

                            tasks.api_schema = Some(filtered_endpoints.clone());

                            info!(
                                "[*] {:?}: Backend Code Unit Testing: Starting the web server to test endpoints...",
                                self.agent.position(),
                            );
                            for endpoint in filtered_endpoints {
                                info!(
                                    "[*] {:?}: Testing endpoint: {}",
                                    self.agent.position(),
                                    endpoint.path
                                );

                                let url = format!("http://127.0.0.1:8080{}", endpoint.path);
                                let status_code =
                                    self.req_client.get(url.to_string()).send().await?.status();

                                if status_code != 200 {
                                    info!(
                                    "[*] {:?}: Failed to fetch the backend endpoint: {}. Further investigation needed...",
                                    self.agent.position(),
                                    endpoint.path
                                );
                                }
                            }

                            let _ = child.kill();

                            let backend_path = format!("{}/{}", path, "api.json");
                            fs::write(backend_path, endpoints)?;
                            info!(
                            "[*] {:?}: Backend testing complete. Results written to 'api.json'...",
                            self.agent.position(),
                        );
                        } else {
                            info!(
                            "[*] {:?}: Backend Code Unit Testing: Failed to build or run the backend project",
                            self.agent.position(),
                        );
                            break;
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
