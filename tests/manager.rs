use autogpt::agents::manager::ManagerGPT;
use tracing::info;
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::test]
async fn test_manager_gpt() {
    let filter = filter::LevelFilter::INFO;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let objective = "Expertise at managing projects at scale";
    let request = "Develop a full stack app that fetches today's weather. Use the axum framework for the backend and Yew rust framework for the frontend.";
    let position = "Manager";

    let mut manager = ManagerGPT::new(objective, position, request);

    manager.execute(true, 3).await;

    info!("{:?}", manager);
}
