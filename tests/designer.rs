#![allow(unused)]

use anyhow::Result;
#[cfg(feature = "img")]
use autogpt::agents::designer::DesignerGPT;
use autogpt::common::utils::{Status, Tasks};
use autogpt::traits::agent::Agent;
use autogpt::traits::functions::Functions;
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::test]
#[cfg(feature = "img")]
async fn test_generate_image_from_text() -> Result<()> {
    let filter = filter::LevelFilter::INFO;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let objective = "Crafts stunning web design layouts";
    let position = "Web Designer";

    let mut designer_agent = DesignerGPT::new(objective, position);

    let mut tasks = Tasks {
        description: "Generate a kanban-style task management board. The board is divided into three columns: To Do, In Progress, and Done. Each column contains a list of tasks. The tasks in the To Do column are prioritized from highest to lowest, with the highest priority task at the top. The tasks in the In Progress column are listed in the order in which they were started. The tasks in the Done column are listed in the order in which they were completed.".into(),
        scope: None,
        urls: None,
        backend_code: None,
        frontend_code: None,
        api_schema: None,
    };

    let _ = designer_agent.generate_image_from_text(&mut tasks).await?;
    assert_eq!(designer_agent.get_agent().memory().len(), 3);
    assert_eq!(designer_agent.get_agent().memory()[0].role, "user");
    assert_eq!(designer_agent.get_agent().memory()[1].role, "assistant");

    Ok(())
}

#[tokio::test]
#[cfg(feature = "img")]
async fn test_execute_agent() -> Result<()> {
    let objective = "Crafts stunning web design layouts";
    let position = "Web Designer";

    let mut designer_agent = DesignerGPT::new(objective, position);

    let mut tasks = Tasks {
        description: "A kanban-style task management board. The board is divided into three columns: To Do, In Progress, and Done. Each column contains a list of tasks. The tasks in the To Do column are prioritized from highest to lowest, with the highest priority task at the top. The tasks in the In Progress column are listed in the order in which they were started. The tasks in the Done column are listed in the order in which they were completed.".into(),
        scope: None,
        urls: None,
        backend_code: None,
        frontend_code: None,
        api_schema: None,
    };

    let _ = designer_agent.execute(&mut tasks, true, 3).await?;
    assert_eq!(designer_agent.get_agent().memory().len(), 3);
    assert_eq!(designer_agent.get_agent().memory()[0].role, "user");
    assert_eq!(designer_agent.get_agent().memory()[1].role, "assistant");

    Ok(())
}
