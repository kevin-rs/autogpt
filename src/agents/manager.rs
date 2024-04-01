use crate::agents::agent::AgentGPT;
use crate::agents::architect::ArchitectGPT;
use crate::agents::backend::BackendGPT;
use crate::agents::frontend::FrontendGPT;
use crate::common::utils::Tasks;
use crate::prompts::manager::MANAGER_PROMPT;
use crate::traits::functions::Functions;
use anyhow::Result;

#[derive(Debug)]
enum AgentType {
    Architect(ArchitectGPT),
    Backend(BackendGPT),
    Frontend(FrontendGPT),
}

impl AgentType {
    async fn execute(&mut self, tasks: &mut Tasks, execute: bool) -> Result<()> {
        match self {
            AgentType::Architect(agent) => agent.execute(tasks, execute).await,
            AgentType::Backend(agent) => agent.execute(tasks, execute).await,
            AgentType::Frontend(agent) => agent.execute(tasks, execute).await,
        }
    }
}

#[derive(Debug)]
#[allow(unused)]
pub struct ManagerGPT {
    agent: AgentGPT,
    tasks: Tasks,
    agents: Vec<AgentType>,
}

impl ManagerGPT {
    pub fn new(objective: &'static str, position: &'static str, request: &'static str) -> Self {
        let agent = AgentGPT::new_borrowed(position, objective);

        let agents: Vec<AgentType> = Vec::new();

        let request = format!("{}\n\nUser Request: {}", MANAGER_PROMPT, request);

        let tasks: Tasks = Tasks {
            description: request.into(),
            scope: None,
            urls: None,
            frontend_code: None,
            backend_code: None,
            api_schema: None,
        };

        Self {
            agent,
            tasks,
            agents,
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
        self.add_agent(AgentType::Backend(BackendGPT::new(
            "Expertise lies in writing backend code for web servers and JSON databases",
            "Backend Developer",
        )));
        self.add_agent(AgentType::Frontend(FrontendGPT::new(
            "Expertise lies in writing frontend code for Yew rust framework",
            "Frontend Developer",
        )));
    }

    pub async fn execute(&mut self, execute: bool) {
        if self.agents.is_empty() {
            self.spawn_default_agents();
        }

        for agent in &mut self.agents {
            let _agent_res = agent.execute(&mut self.tasks, execute).await;
        }
    }
}