use autogpt::agents::optimizer::OptimizerGPT;
use autogpt::common::utils::{Status, Tasks};
use autogpt::traits::agent::Agent;
use autogpt::traits::functions::Functions;
use std::fs;
use std::{fs::File, io::Write, path::Path};
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::test]
async fn test_optimizer_gpt_execute() {
    let filter = filter::LevelFilter::INFO;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let objective = "Optimize and modularize backend code";
    let position = "OptimizerGPT";
    let language = "rust";

    let mut optimizer_agent = OptimizerGPT::new(objective, position, language);

    let workspace = optimizer_agent.workspace.to_string();
    let workspace_path = Path::new(&workspace);
    let src_path = workspace_path.join("src");
    if !src_path.exists() {
        std::fs::create_dir_all(&src_path).unwrap();
    }

    let main_file_path = src_path.join("main.rs");
    let mut file = File::create(&main_file_path).unwrap();
    let file_content = r#"
fn main() {
    println!("Hello, world!");
}
"#;
    file.write_all(file_content.as_bytes()).unwrap();

    let mut tasks = Tasks {
        description: "Refactor backend code for better modularization".into(),
        scope: None,
        urls: None,
        frontend_code: None,
        backend_code: None,
        api_schema: None,
    };

    optimizer_agent
        .execute(&mut tasks, true, false, 3)
        .await
        .unwrap();

    assert_eq!(optimizer_agent.get_agent().memory().len(), 4);
    assert_eq!(optimizer_agent.get_agent().memory()[0].role, "user");
    assert_eq!(optimizer_agent.get_agent().memory()[1].role, "assistant");

    assert!(tasks.backend_code.is_some());

    assert!(workspace_path.exists());

    let main_file_path_check = workspace_path.join("src").join("main.rs");
    assert!(main_file_path_check.exists());

    assert_eq!(optimizer_agent.get_agent().status(), &Status::Completed);
}

#[tokio::test]
async fn test_optimizer_gpt_save_module() {
    let objective = "Optimize and modularize backend code";
    let position = "OptimizerGPT";
    let language = "rust";

    let optimizer_agent = OptimizerGPT::new(objective, position, language);

    let workspace = optimizer_agent.workspace.to_string();
    let workspace_path = Path::new(&workspace);
    let src_path = workspace_path.join("src");
    if !src_path.exists() {
        std::fs::create_dir_all(&src_path).unwrap();
    }

    let main_file_path = src_path.join("main.rs");
    let mut file = File::create(&main_file_path).unwrap();
    let file_content = r#"
fn main() {
    println!("Hello, world!");
}
"#;
    file.write_all(file_content.as_bytes()).unwrap();

    let filename = "module.rs";
    let content = "// This is a refactored module";

    let result = optimizer_agent.save_module(filename, content);
    assert!(result.is_ok());

    let module_path = workspace_path.join(filename);
    assert!(module_path.exists());

    let saved_content = std::fs::read_to_string(module_path).unwrap();
    assert_eq!(saved_content, content);
}

#[tokio::test]
async fn test_generate_and_track() {
    let objective = "Optimize and modularize backend code";
    let position = "OptimizerGPT";
    let language = "rust";

    let mut optimizer_agent = OptimizerGPT::new(objective, position, language);

    let workspace = optimizer_agent.workspace.to_string();
    let workspace_path = Path::new(&workspace);
    let src_path = workspace_path.join("src");
    if !src_path.exists() {
        std::fs::create_dir_all(&src_path).unwrap();
    }

    let main_file_path = src_path.join("main.rs");
    let mut file = File::create(&main_file_path).unwrap();
    let file_content = r#"
fn main() {
    println!("Hello, world!");
}
"#;
    file.write_all(file_content.as_bytes()).unwrap();

    let request = "Refactor the following function to improve readability and modularity.";
    let response = optimizer_agent.generate_and_track(request).await;

    assert!(!response.is_empty());

    assert_eq!(optimizer_agent.get_agent().memory().len(), 1);
    assert_eq!(optimizer_agent.get_agent().memory()[0].role, "assistant");
    let workspace = optimizer_agent.workspace.to_string();
    let workspace_path = Path::new(&workspace);
    let workspace_path = Path::new(workspace_path);
    if workspace_path.exists() {
        fs::remove_dir_all(workspace_path).unwrap();
    }
}
