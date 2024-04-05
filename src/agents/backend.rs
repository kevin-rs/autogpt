//! # `BackendGPT` agent.
//!
//! This module provides functionality for generating backend code for web servers
//! and JSON databases based on prompts using Gemini API. The `BackendGPT` agent
//! understands user requirements and produces code snippets in various programming
//! languages commonly used for backend development.
//!
//! # Example - Generating backend code:
//!
//! ```rust
//! use autogpt::agents::backend::BackendGPT;
//! use autogpt::common::utils::Tasks;
//! use autogpt::traits::functions::Functions;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut backend_agent = BackendGPT::new(
//!         "Generate backend code",
//!         "Backend Developer",
//!         "rust",
//!     );
//!
//!     let mut tasks = Tasks {
//!         description: "Create REST API endpoints for user authentication".into(),
//!         scope: None,
//!         urls: None,
//!         frontend_code: None,
//!         backend_code: None,
//!         api_schema: None,
//!     };
//!
//!     if let Err(err) = backend_agent.execute(&mut tasks, true, 3).await {
//!         eprintln!("Error executing backend tasks: {:?}", err);
//!     }
//! }
//! ```
//!

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
use tracing::{debug, error, info};
use webbrowser::{open_browser_with_options, Browser, BrowserOptions};

/// Struct representing a BackendGPT, which manages backend development tasks using GPT.
#[derive(Debug, Clone)]
pub struct BackendGPT {
    /// Represents the workspace directory path for BackendGPT.
    workspace: Cow<'static, str>,
    /// Represents the GPT agent responsible for handling backend tasks.
    agent: AgentGPT,
    /// Represents a Gemini client for interacting with Gemini API.
    client: Client,
    /// Represents a client for making HTTP requests.
    req_client: ReqClient,
    /// Represents the bugs found in the codebase, if any.
    bugs: Option<Cow<'static, str>>,
    /// Represents the programming language used for backend development.
    language: &'static str,
    /// Represents the number of bugs found in the codebase.
    nb_bugs: u64,
}

impl BackendGPT {
    /// Constructor function to create a new instance of `BackendGPT`.
    ///
    /// # Arguments
    ///
    /// * `objective` - Objective description for `BackendGPT`.
    /// * `position` - Position description for `BackendGPT`.
    /// * `language` - Programming language used for backend development.
    ///
    /// # Returns
    ///
    /// (`BackendGPT`): A new instance of `BackendGPT`.
    ///
    /// # Business Logic
    ///
    /// - Constructs the workspace directory path for `BackendGPT`.
    /// - Initializes backend projects based on the specified language.
    /// - Initializes the GPT agent with the given objective and position.
    /// - Creates clients for interacting with Gemini API and making HTTP requests.
    ///
    pub fn new(objective: &'static str, position: &'static str, language: &'static str) -> Self {
        let workspace = var("AUTOGPT_WORKSPACE")
            .unwrap_or("workspace/".to_string())
            .to_owned()
            + "backend";

        match language {
            "rust" => {
                let cargo_new = Command::new("cargo").arg("new").arg(&workspace).spawn();

                match cargo_new {
                    Ok(_) => debug!("Cargo project initialized successfully!"),
                    Err(e) => error!("Error initializing Cargo project: {}", e),
                };
                match fs::write(workspace.to_string() + "src/template.rs", "") {
                    Ok(_) => debug!("File 'template.rs' created successfully!"),
                    Err(e) => error!("Error creating file 'template.rs': {}", e),
                };
            }
            "python" => {
                match fs::write(workspace.to_string() + "/main.py", "") {
                    Ok(_) => debug!("File 'main.py' created successfully!"),
                    Err(e) => error!("Error creating file 'main.py': {}", e),
                };
                match fs::write(workspace.to_string() + "/template.py", "") {
                    Ok(_) => debug!("File 'template.py' created successfully!"),
                    Err(e) => error!("Error creating file 'template.py': {}", e),
                };
            }
            "javascript" => {
                let npx_install = Command::new("npx")
                    .arg("create-react-app")
                    .arg(&workspace)
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
                };
                match fs::write(workspace.to_string() + "src/template.js", "") {
                    Ok(_) => debug!("File 'template.js' created successfully!"),
                    Err(e) => error!("Error creating file 'template.js': {}", e),
                };
            }
            _ => panic!("Unsupported language, consider open an Issue/PR"),
        };

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
            language,
            nb_bugs: 0,
        }
    }

    /// Asynchronously generates backend code based on tasks.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to tasks to be processed.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): Result containing the generated backend code.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in generating the backend code.
    ///
    /// # Business Logic
    ///
    /// - Determines the file path based on the specified language.
    /// - Reads the template code from the specified file.
    /// - Constructs a request based on the template code and project description.
    /// - Sends the request to the Gemini API to generate content.
    /// - Writes the generated backend code to the appropriate file.
    /// - Updates tasks and agent status accordingly.
    ///
    pub async fn generate_backend_code(&mut self, tasks: &mut Tasks) -> Result<String> {
        let path = &self.workspace;

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

        debug!("[*] {:?}: {:?}", self.agent.position(), full_path);

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
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }

    /// Asynchronously improves existing backend code based on tasks.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to tasks to be processed.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): Result containing the improved backend code.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in improving the backend code.
    ///
    /// # Business Logic
    ///
    /// - Constructs a request based on the existing backend code and project description.
    /// - Sends the request to the Gemini API to generate content.
    /// - Writes the improved backend code to the appropriate file.
    /// - Updates tasks and agent status accordingly.
    ///
    pub async fn improve_backend_code(&mut self, tasks: &mut Tasks) -> Result<String> {
        let path = &self.workspace;

        let request: String = format!(
            "{}\n\nCode Template: {}\nProject Description: {}",
            IMPROVED_WEBSERVER_CODE_PROMPT,
            tasks.clone().backend_code.unwrap_or_default(),
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

        debug!("[*] {:?}: {:?}", self.agent.position(), backend_path);

        fs::write(backend_path, gemini_response.clone())?;

        tasks.backend_code = Some(gemini_response.clone().into());

        self.agent.update(Status::Completed);
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }

    /// Asynchronously fixes bugs in the backend code based on tasks.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to tasks to be processed.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): Result containing the fixed backend code.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in fixing the backend code bugs.
    ///
    /// # Business Logic
    ///
    /// - Constructs a request based on the buggy backend code and project description.
    /// - Sends the request to the Gemini API to generate content for fixing bugs.
    /// - Writes the fixed backend code to the appropriate file.
    /// - Updates tasks and agent status accordingly.
    ///
    pub async fn fix_code_bugs(&mut self, tasks: &mut Tasks) -> Result<String> {
        let path = var("AUTOGPT_WORKSPACE")
            .unwrap_or("workspace/backend".to_string())
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
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }

    /// Asynchronously retrieves routes JSON from the backend code.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): Result containing the routes JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in retrieving routes JSON.
    ///
    /// # Business Logic
    ///
    /// - Reads the backend code from the appropriate file.
    /// - Constructs a request with the backend code.
    /// - Sends the request to the Gemini API to generate content.
    /// - Updates agent status accordingly.
    ///
    pub async fn get_routes_json(&mut self) -> Result<String> {
        let path = &self.workspace;

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

        debug!("[*] {:?}: {:?}", self.agent.position(), full_path);

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
        debug!("[*] {:?}: {:?}", self.agent.position(), self.agent);

        Ok(gemini_response)
    }

    /// Accessor method to retrieve the agent associated with BackendGPT.
    ///
    /// # Returns
    ///
    /// (`&AgentGPT`): Reference to the agent.
    ///
    /// # Business Logic
    ///
    /// - Provides access to the agent associated with the BackendGPT instance.
    ///
    pub fn agent(&self) -> &AgentGPT {
        &self.agent
    }

    /// Updates the bugs found in the codebase.
    ///
    /// # Arguments
    ///
    /// * `bugs` - Optional description of bugs found in the codebase.
    ///
    /// # Business Logic
    ///
    /// - Updates the bugs field with the provided description.
    ///
    pub fn update_bugs(&mut self, bugs: Option<Cow<'static, str>>) {
        self.bugs = bugs;
    }
}

/// Implementation of the trait `Functions` for `BackendGPT`.
/// Contains additional methods related to backend tasks.
///
/// This trait provides methods for:
///
/// - Retrieving the agent associated with `BackendGPT`.
/// - Executing tasks asynchronously.
///
/// # Business Logic
///
/// - Provides access to the agent associated with the `BackendGPT` instance.
/// - Executes tasks asynchronously based on the current status of the agent.
/// - Handles task execution including code generation, bug fixing, and testing.
/// - Manages retries and error handling during task execution.
///
impl Functions for BackendGPT {
    /// Retrieves a reference to the agent.
    ///
    /// # Returns
    ///
    /// (`&AgentGPT`): A reference to the agent.
    ///
    fn get_agent(&self) -> &AgentGPT {
        &self.agent
    }

    /// Asynchronously executes tasks associated with BackendGPT.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to tasks to be executed.
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
    /// - Executes tasks asynchronously based on the current status of the agent.
    /// - Handles task execution including code generation, bug fixing, and testing.
    /// - Manages retries and error handling during task execution.
    ///
    async fn execute(&mut self, tasks: &mut Tasks, execute: bool, max_tries: u64) -> Result<()> {
        info!(
            "[*] {:?}: Executing tasks: {:?}",
            self.agent.position(),
            tasks.clone()
        );

        let path = &self.workspace.to_string();

        // TODO: add a func argument for webbrowser
        if !execute {
            let _ = open_browser_with_options(
                Browser::Default,
                "http://127.0.0.1:8000/docs",
                BrowserOptions::new().with_suppress_output(false),
            );
        }

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
                                    .current_dir(path);
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
                                        .current_dir(path)
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
                                    .current_dir(path)
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
                                    .arg("src/index.js")
                                    .current_dir(path)
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
                            fs::write(&backend_path, endpoints)?;
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
