use autogpt::agents::architect::ArchitectGPT;
use autogpt::prelude::*;
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::test]
async fn test_autogpt_zero_agents() {
    let filter = filter::LevelFilter::INFO;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let autogpt = AutoGPT::default().build().expect("Failed to build AutoGPT");

    // No agents to run.
    let result = autogpt.run().await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_autogpt_with_agents() {
    let objective = "Creates innovative website designs and user experiences";
    let position = "Lead UX/UI Designer";

    let agent = ArchitectGPT::new(objective, position).await;

    let autogpt = AutoGPT::default()
        .with(agents![agent])
        .build()
        .expect("Failed to build AutoGPT");

    let result = autogpt.run().await;

    assert!(result.is_ok());
}
