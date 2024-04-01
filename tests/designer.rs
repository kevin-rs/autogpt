#![allow(unused)]

use anyhow::Result;
use autogpt::agents::designer::DesignerGPT;
use autogpt::common::utils::{Status, Tasks};
use autogpt::traits::agent::Agent;
use autogpt::traits::functions::Functions;
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::test]
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
        description: "Generate a minimalist layout with a clean interface showing the forecast for the next week, with options to switch between Celsius and Fahrenheit.".into(),
        scope: None,
        urls: None,
        backend_code: None,
        frontend_code: None,
        api_schema: None,
    };

    // let _ = designer_agent.generate_image_from_text(&mut tasks).await?;

    // assert_eq!(designer_agent.get_agent().status(), &Status::Completed);

    Ok(())
}

#[tokio::test]
async fn test_generate_text_from_image() -> Result<()> {
    let objective = "Crafts stunning web design layouts";
    let position = "Web Designer";

    let mut designer_agent = DesignerGPT::new(objective, position);

    let _ = designer_agent
        .generate_text_from_image("data/img.png")
        .await?;

    Ok(())
}
