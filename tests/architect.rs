use autogpt::agents::architect::ArchitectGPT;
use autogpt::common::utils::{Scope, Status, Tasks};
use autogpt::traits::agent::Agent;
use autogpt::traits::functions::Functions;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::test]
async fn test_get_scope() {
    tracing_subscriber::registry().with(fmt::layer()).init();
    let objective = "Creates innovative website designs and user experiences";
    let position = "Lead UX/UI Designer";

    let mut architect_agent = ArchitectGPT::new(objective, position);

    let mut tasks = Tasks {
        description: "Create a blog platform for publishing articles and comments.".into(),
        scope: None,
        urls: None,
        backend_code: None,
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

    assert_eq!(architect_agent.agent().status(), &Status::Completed);
}

#[tokio::test]
async fn test_get_urls() {
    let objective = "Creates innovative website designs and user experiences";
    let position = "Lead UX/UI Designer";

    let mut architect_agent = ArchitectGPT::new(objective, position);

    let mut tasks = Tasks {
        description: "Create a weather forecast website for global cities.".into(),
        scope: Some(Scope {
            crud: true,
            auth: false,
            external: true,
        }),
        urls: None,
        backend_code: None,
        api_schema: None,
    };

    let _ = architect_agent.get_urls(&mut tasks).await;

    assert!(tasks.urls.unwrap().len() >= 1);
    assert_eq!(architect_agent.agent().status(), &Status::InUnitTesting);
}

#[tokio::test]
async fn test_architect_agent() {
    let objective = "Creates innovative website designs and user experiences";
    let position = "Lead UX/UI Designer";

    let mut architect_agent = ArchitectGPT::new(objective, position);

    let mut tasks = Tasks {
        description: "Create a weather forecast website for global cities.".into(),
        scope: Some(Scope {
            crud: true,
            auth: false,
            external: true,
        }),
        urls: None,
        backend_code: None,
        api_schema: None,
    };

    architect_agent
        .execute(&mut tasks)
        .await
        .expect("Unable to execute Solutions Architect Agent");

    assert!(tasks.scope != None);
    assert!(tasks.urls.is_some());

    dbg!(tasks);
}
