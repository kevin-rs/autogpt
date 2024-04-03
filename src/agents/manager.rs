use crate::agents::agent::AgentGPT;
use crate::agents::architect::ArchitectGPT;
use crate::agents::backend::BackendGPT;
use crate::agents::designer::DesignerGPT;
use crate::agents::frontend::FrontendGPT;
use crate::common::utils::strip_code_blocks;
use crate::common::utils::Tasks;
use crate::prompts::manager::{FRAMEWORK_MANAGER_PROMPT, LANGUAGE_MANAGER_PROMPT, MANAGER_PROMPT};
use crate::traits::agent::Agent;
use crate::traits::functions::Functions;
use anyhow::Result;
use gems::Client;
use std::env::var;
use tracing::info;

#[derive(Debug, Clone)]
enum AgentType {
    Architect(ArchitectGPT),
    Backend(BackendGPT),
    Frontend(FrontendGPT),
    Designer(DesignerGPT),
}

impl AgentType {
    async fn execute(&mut self, tasks: &mut Tasks, execute: bool, max_tries: u64) -> Result<()> {
        match self {
            AgentType::Architect(agent) => agent.execute(tasks, execute, max_tries).await,
            AgentType::Backend(agent) => agent.execute(tasks, execute, max_tries).await,
            AgentType::Frontend(agent) => agent.execute(tasks, execute, max_tries).await,
            AgentType::Designer(agent) => agent.execute(tasks, execute, max_tries).await,
        }
    }
    fn position(&self) -> String {
        match self {
            AgentType::Architect(agent) => agent.agent().position().to_string(),
            AgentType::Backend(agent) => agent.agent().position().to_string(),
            AgentType::Frontend(agent) => agent.agent().position().to_string(),
            _ => "Any".to_string(),
        }
    }
}

#[derive(Debug)]
#[allow(unused)]
pub struct ManagerGPT {
    agent: AgentGPT,
    tasks: Tasks,
    language: &'static str,
    agents: Vec<AgentType>,
    client: Client,
}

impl ManagerGPT {
    pub fn new(
        objective: &'static str,
        position: &'static str,
        request: &'static str,
        language: &'static str,
    ) -> Self {
        let agent = AgentGPT::new_borrowed(objective, position);

        let agents: Vec<AgentType> = Vec::new();

        // let request = format!("{}\n\nUser Request: {}", MANAGER_PROMPT, request);

        let tasks: Tasks = Tasks {
            description: request.into(),
            scope: None,
            urls: None,
            frontend_code: None,
            backend_code: None,
            api_schema: None,
        };

        let model = var("GEMINI_MODEL")
            .unwrap_or("gemini-pro".to_string())
            .to_owned();

        let api_key = var("GEMINI_API_KEY").unwrap_or_default().to_owned();
        let client = Client::new(&api_key, &model);

        Self {
            agent,
            tasks,
            language,
            agents,
            client,
        }
    }

    fn add_agent(&mut self, agent: AgentType) {
        self.agents.push(agent);
    }

    fn spawn_default_agents(&mut self) {
        self.add_agent(AgentType::Architect(ArchitectGPT::new(
            "Creates innovative website designs and user experiences",
            "Lead UX/UI Designer",
        )));
        self.add_agent(AgentType::Designer(DesignerGPT::new(
            "Creates innovative website designs and user experiences",
            "Web wireframes and web UIs",
        )));
        self.add_agent(AgentType::Backend(BackendGPT::new(
            "Expertise lies in writing backend code for web servers and JSON databases",
            "Backend Developer",
            self.language,
        )));
        self.add_agent(AgentType::Frontend(FrontendGPT::new(
            "Expertise lies in writing frontend code for Yew rust framework",
            "Frontend Developer",
            self.language,
        )));
    }

    pub async fn execute_prompt(&self, prompt: String) -> Result<String> {
        let gemini_response: String = match self.client.clone().generate_content(&prompt).await {
            Ok(response) => strip_code_blocks(&response),
            Err(_err) => Default::default(),
        };

        Ok(gemini_response)
    }

    pub async fn execute(&mut self, execute: bool, max_tries: u64) -> Result<()> {
        info!(
            "[*] {:?}: Executing task: {:?}",
            self.agent.position(),
            self.tasks.description.clone()
        );

        let language_request: String = format!(
            "{}\n\nUser Request: {}",
            LANGUAGE_MANAGER_PROMPT,
            self.tasks.description.clone()
        );

        let framework_request: String = format!(
            "{}\n\nUser Request: {}",
            FRAMEWORK_MANAGER_PROMPT,
            self.tasks.description.clone()
        );

        let language = self.execute_prompt(language_request).await?;
        let framework = self.execute_prompt(framework_request).await?;

        if self.agents.is_empty() {
            self.spawn_default_agents();
        }

        for mut agent in self.agents.clone() {
            let request_prompt = format!("{}\n\n\n\nUser Request: {}\n\nAgent Role: {}\nProgramming Language: {}\nFramework: {}\n",
                MANAGER_PROMPT, self.tasks.description.clone(), agent.position(),language, framework);

            let request = self.execute_prompt(request_prompt).await?;

            self.tasks = Tasks {
                description: request.into(),
                scope: None,
                urls: None,
                frontend_code: None,
                backend_code: None,
                api_schema: None,
            };

            let _agent_res = agent.execute(&mut self.tasks, execute, max_tries).await;
        }

        Ok(())
    }
}
