#![allow(unused)]

use autogpt::agents::backend::BackendGPT;
use autogpt::common::utils::{Scope, Tasks};
use autogpt::traits::agent::Agent;
use autogpt::traits::functions::Functions;
use std::fs;
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::test]
async fn test_generate_backend_code() {
    let filter = filter::LevelFilter::INFO;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let objective = "Expertise lies in writing backend code for web servers and JSON databases";
    let position = "Backend Developer";

    let mut backend_gpt = BackendGPT::new(objective, position, "python");
    let mut tasks = Tasks {
        description: "Generate a todo crud app using python FastAPI.".into(),
        scope: Some(Scope {
            crud: true,
            auth: true,
            external: false,
        }),
        urls: None,
        frontend_code: None,
        backend_code: None,
        api_schema: None,
    };

    let result = backend_gpt.generate_backend_code(&mut tasks).await;
    assert_eq!(backend_gpt.get_agent().memory().len(), 2);
    assert_eq!(backend_gpt.get_agent().memory()[0].role, "user");
    assert_eq!(backend_gpt.get_agent().memory()[1].role, "assistant");

    assert!(result.is_ok());
    assert!(tasks.backend_code.is_some());
}

#[tokio::test]
async fn test_improve_backend_code() {
    let objective = "Expertise lies in writing backend code for web servers and JSON databases";
    let position = "Backend Developer";

    let mut backend_gpt = BackendGPT::new(objective, position, "python");
    let mut tasks = Tasks {
        description: "Generate a todo crud app using Axum.".into(),
        scope: Some(Scope {
            crud: true,
            auth: true,
            external: false,
        }),
        urls: None,
        frontend_code: None,
        backend_code: None,
        api_schema: None,
    };
    tasks.backend_code = Some(
        r#"
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Deserialize)]
struct CreateMessageRequest {
    message: String,
}

type Messages = Arc<Mutex<HashMap<String, String>>>;

async fn create_message(
    Json(payload): Json<CreateMessageRequest>,
    messages: Messages,
) -> (StatusCode, Json<serde_json::Value>) {
    let id = uuid::Uuid::new_v4().to_string();
    messages.lock().await.insert(id.clone(), payload.message);
    (StatusCode::OK, Json(json!({"id": id})))
}

async fn get_message(
    Json(payload): Json<HashMap<String, String>>,
    messages: Messages,
) -> (StatusCode, Json<serde_json::Value>) {
    match messages.lock().await.get(&payload["id"]) {
        Some(message) => (StatusCode::OK, Json(json!({"message": message}))),
        None => (StatusCode::NOT_FOUND, Json(json!({"error": "Message not found"}))),
    }
}

#[tokio::main]
async fn main() {
    let messages = Arc::new(Mutex::new(HashMap::new()));
    let app = Router::new()
        .route("/create-message", post(create_message))
        .route("/get-message", get(get_message))
        .with_state(messages);

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
"#
        .into(),
    );

    let result = backend_gpt.improve_backend_code(&mut tasks).await;
    assert_eq!(backend_gpt.get_agent().memory().len(), 2);
    assert_eq!(backend_gpt.get_agent().memory()[0].role, "user");
    assert_eq!(backend_gpt.get_agent().memory()[1].role, "assistant");

    assert!(result.is_ok());
    assert!(tasks.backend_code.is_some());
}

#[tokio::test]
async fn test_fix_code_bugs() {
    let objective = "Expertise lies in writing backend code for web servers and JSON databases";
    let position = "Backend Developer";

    let mut backend_gpt = BackendGPT::new(objective, position, "python");
    let mut tasks = Tasks {
        description: "Generate a todo crud app using Axum.".into(),
        scope: Some(Scope {
            crud: true,
            auth: true,
            external: false,
        }),
        urls: None,
        frontend_code: None,
        backend_code: None,
        api_schema: None,
    };
    tasks.backend_code = Some(
        r#"use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Deserialize)]
struct CreateMessageRequest {
    message: String,
}

type Messages = Arc<Mutex<HashMap<String, String>>>;

async fn create_message(
    Json(payload): Json<CreateMessageRequest>,
    messages: Messages,
) -> (StatusCode, Json<serde_json::Value>) {
    let id = uuid::Uuid::new_v4().to_string();
    messages.lock().await.insert(id.clone(), payload.message);
    (StatusCode::OK, Json(json!({"id": id})))
}

async fn get_message(
    Json(payload): Json<HashMap<String, String>>,
    messages: Messages,
) -> (StatusCode, Json<serde_json::Value>) {
    match messages.lock().await.get(&payload["id"]) {
        Some(message) => (StatusCode::OK, Json(json!({"message": message}))),
        None => (StatusCode::NOT_FOUND, Json(json!({"error": "Message not found"}))),
    }
}

#[tokio::main]
async fn main() {
    let messages = Arc::new(Mutex::new(HashMap::new()));
    let app = Router::new()
        .route("/create-message", post(create_message))
        .route("/get-message", get(get_message))
        .with_state(messages);

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
"#
        .into(),
    );
    backend_gpt.update_bugs(Some(r#"
error[E0433]: failed to resolve: could not find `Server` in `axum`
  --> src/main.rs:46:11
   |
46 |     axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
   |           ^^^^^^ could not find `Server` in `axum`

error[E0277]: the trait bound `fn(Json<CreateMessageRequest>, Arc<tokio::sync::Mutex<HashMap<std::string::String, std::string::String>>>) -> impl std::future::Future<Output = (StatusCode, Json<Value>)> {create_message}: Handler<_, _>` is not satisfied
   --> src/main.rs:42:40
    |
42  |         .route("/create-message", post(create_message))
    |                                   ---- ^^^^^^^^^^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(Json<CreateMessageRequest>, Arc<Mutex<HashMap<String, String>>>) -> ... {create_message}`
    |                                   |
    |                                   required by a bound introduced by this call


error[E0277]: the trait bound `fn(Json<HashMap<std::string::String, std::string::String>>, Arc<tokio::sync::Mutex<HashMap<std::string::String, std::string::String>>>) -> impl std::future::Future<Output = (StatusCode, Json<Value>)> {get_message}: Handler<_, _>` is not satisfied
   --> src/main.rs:43:36
    |
43  |         .route("/get-message", get(get_message))
    |                                --- ^^^^^^^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(Json<HashMap<String, String>>, Arc<Mutex<HashMap<String, String>>>) -> ... {get_message}`
    |                                |
    |                                required by a bound introduced by this call
    |


385 | top_level_handler_fn!(get, GET);
    | ^^^^^^^^^^^^^^^^^^^^^^---^^^^^^
    | |                     |
    | |                     required by a bound in this function
    | required by this bound in `get`
    = note: this error originates in the macro `top_level_handler_fn` (in Nightly builds, run with -Z macro-backtrace for more info)

Some errors have detailed explanations: E0277, E0433.

"#.into()));

    let result = backend_gpt.fix_code_bugs(&mut tasks).await;
    assert_eq!(backend_gpt.get_agent().memory().len(), 2);
    assert_eq!(backend_gpt.get_agent().memory()[0].role, "user");
    assert_eq!(backend_gpt.get_agent().memory()[1].role, "assistant");

    assert!(result.is_ok());
    assert!(tasks.backend_code.is_some());
}

#[tokio::test]
async fn test_get_routes_json() {
    let objective = "Expertise lies in writing backend code for web servers and JSON databases";
    let position = "Backend Developer";

    let mut backend_gpt = BackendGPT::new(objective, position, "python");

    let result = backend_gpt.get_routes_json().await;

    assert!(result.is_ok());
    fs::write::<&str, String>("workspace/backend/api.json", result.unwrap()).unwrap();
}

#[tokio::test]
async fn tests_backend_dev_one() {
    let objective = "Expertise lies in writing backend code for web servers and JSON databases";
    let position = "Backend Developer";

    let mut backend_gpt = BackendGPT::new(objective, position, "python");

    let mut tasks = Tasks {
        description: "Generate a todo crud app using python FastAPI.".into(),
        scope: Some(Scope {
            crud: true,
            auth: true,
            external: true,
        }),
        urls: Some(vec![
            "https://kevin-rs.dev/products".into(),
            "https://kevin-rs.dev/cart".into(),
        ]),
        frontend_code: None,
        backend_code: None,
        api_schema: None,
    };

    // backend_gpt.execute(&mut tasks, true, 3).await.unwrap();
}
