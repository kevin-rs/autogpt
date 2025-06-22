use autogpt::prelude::*;
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::test]
async fn test_autogpt() {
    let filter = filter::LevelFilter::INFO;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let mut autogpt = AutoGPTBuilder::default()
        .build()
        .expect("Failed to build AutoGPT");

    let msg = Message::default();

    let result = autogpt.run(vec![msg.clone()]).await;

    assert!(result.is_ok());
}
