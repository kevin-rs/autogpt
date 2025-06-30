use autogpt::agents::architect::ArchitectGPT;
use autogpt::common::utils::{Scope, Status, Task};
use autogpt::traits::agent::Agent;
use autogpt::traits::functions::{AsyncFunctions, Functions};
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::test]
async fn test_get_scope() {
    let filter = filter::LevelFilter::INFO;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let objective = "Creates innovative website designs and user experiences";
    let position = "Lead UX/UI Designer";

    let mut architect_agent = ArchitectGPT::new(objective, position).await;

    let mut tasks = Task {
        description: "Create a blog platform for publishing articles and comments.".into(),
        scope: None,
        urls: None,
        backend_code: None,
        frontend_code: None,
        api_schema: None,
    };

    let scope = architect_agent.get_scope(&mut tasks).await.unwrap();

    assert_eq!(
        scope,
        Scope {
            crud: true,
            auth: true,
            external: false,
        }
    );

    assert_eq!(architect_agent.get_agent().status(), &Status::Completed);
}

#[tokio::test]
async fn test_get_urls() {
    let objective = "Creates innovative website designs and user experiences";
    let position = "Lead UX/UI Designer";

    let mut architect_agent = ArchitectGPT::new(objective, position).await;

    let mut tasks = Task {
        description: "Create a weather forecast website for global cities.".into(),
        scope: Some(Scope {
            crud: true,
            auth: false,
            external: true,
        }),
        urls: Some(Vec::new()),
        frontend_code: None,
        backend_code: None,
        api_schema: None,
    };

    let _ = architect_agent.get_urls(&mut tasks).await;
    // 1 msg from user and 1 msg from assistant -> 2
    assert_eq!(architect_agent.get_agent().memory().len(), 2);
    assert_eq!(architect_agent.get_agent().memory()[0].role, "user");
    assert_eq!(architect_agent.get_agent().memory()[1].role, "assistant");

    assert!(!tasks.urls.unwrap().is_empty());
    assert_eq!(architect_agent.get_agent().status(), &Status::InUnitTesting);
}

#[tokio::test]
async fn test_architect_agent() {
    let objective = "Creates innovative website designs and user experiences";
    let position = "Lead UX/UI Designer";

    let mut architect_agent = ArchitectGPT::new(objective, position).await;

    let mut tasks = Task {
        description: "Create a weather forecast website for global cities.".into(),
        scope: Some(Scope {
            crud: true,
            auth: false,
            external: true,
        }),
        urls: None,
        frontend_code: None,
        backend_code: None,
        api_schema: None,
    };

    architect_agent
        .execute(&mut tasks, true, false, 1)
        .await
        .unwrap();
    assert!(architect_agent.get_agent().memory().len() >= 3);
    assert_eq!(architect_agent.get_agent().memory()[0].role, "user");
    assert_eq!(architect_agent.get_agent().memory()[1].role, "assistant");

    assert!(tasks.scope.is_some());
    // assert!(tasks.urls.is_some());

    dbg!(tasks);
}
