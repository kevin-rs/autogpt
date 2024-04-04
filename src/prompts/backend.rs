pub(crate) const WEBSERVER_CODE_PROMPT: &str = r#"
Your task is to generate backend code for a web server. Generate all your code inside one single file/module.
Don't assume there are module outside the main file, like "from .database import engine, get_db". Generate their
implementation within the same module. So don't import anything from the current directory. Dont generate "from .module import function"

Instructions:
- The user will provide a project description and a code template for a website backend build.
- The backend code provided is only an example. Modify it as needed to match the project description.
- Write functions that make sense for the user's request if required.
- You can use the provided libraries: serde, serde_json, tokio, axum if the selected language is Rust.
- You should only output the code, nothing else.
- You should remove all backticks surrounding the source code. Remove the first and last lines(remove "```").

Example:

Input:
  Project Description: "Build a RESTful API for managing tasks."
  Code Template: "async fn create_task() -> impl IntoResponse{}"

Output:
#[derive(Debug, Deserialize)]
struct ItemRequest {
    input_text: String,
}

  async fn create_task(Json(request): Json<ItemRequest>) -> impl IntoResponse {
    Ok(())
  }

async fn generate_content(
    Extension(mut client): Extension<Client>,
    Json(request): Json<GenerateContentRequest>,
) -> impl IntoResponse {
    match client.generate_content(&request.input_text).await {
        Ok(response) => response,
        Err(error) => error.to_string(),
    }
}
"#;

pub(crate) const IMPROVED_WEBSERVER_CODE_PROMPT: &str = r#"
Your task is to improve the provided backend code for a web server. Generate all your code inside one single file/module.
Don't assume there are module outside the main file, like "from .database import engine, get_db". Generate their
implementation within the same module. So don't import anything from the current directory. Dont generate "from .module import function"

Instructions:
- The user will provide a project description and a code template for a website backend build.
- Tasks:
  1. Fix any bugs in the code and add minor additional functionality.
  2. Ensure compliance with all backend requirements specified in the project description. Add any missing features.
  3. Write the code without any commentary.
- You can use the provided libraries: serde, serde_json, tokio, axum.
- You should only output the code, nothing else.
"#;

pub(crate) const FIX_CODE_PROMPT: &str = r#"
Your task is to fix the code with removed bugs. Generate all your code inside one single file/module.
Don't assume there are module outside the main file, like "from .database import engine, get_db". Generate their
implementation within the same module. So don't import anything from the current directory. Dont generate "from .module import function"

Instructions:
- The user will provide a broken code and the identified errors or bugs.
- Your task is to fix the bugs in the code.
- You should only output the new and improved code, without any commentary.
- You should remove all backticks surrounding the source code. Remove the first and last lines(remove "```").
"#;

pub(crate) const API_ENDPOINTS_PROMPT: &str = r#"
Generate JSON schema representations for the specified REST API endpoints.

Instructions:
- You will be given Rust web server code utilizing a backend web framework.
- Your task is to produce JSON schema descriptions for each URL endpoint along with their respective data types.
- Analyze the provided code to extract information for the following object keys:
  - "route": Represents the URL path of the endpoint.
  - "dynamic": Indicates whether the route is dynamic.
  - "method": Denotes the HTTP method used by the endpoint.
  - "body": Describes the structure of the request body for POST method requests.
  - "response": Specifies the expected output based on the provided structs and functions.
- Ensure that all keys in the JSON schema are represented as strings, including boolean values enclosed in double quotes.
- The generated output should strictly consist of the JSON schema representations without any additional commentary.
- You should remove all backticks surrounding the source code. Remove the first and last lines(remove "```").

Example:

User Request:

use axum::{
    extract::Extension,
    http::{HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::method_routing::post,
    Json, Router,
};
use dotenv_codegen::dotenv;
use gems::Client;
use http::header::CONTENT_TYPE;
use std::time::Duration;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let allowed_origins = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::OPTIONS,
            Method::PUT,
            Method::DELETE,
        ])
        .allow_headers([CONTENT_TYPE])
        .max_age(Duration::from_secs(3600));

    let api_key = dotenv!("GEMINI_API_KEY");
    let model = dotenv!("GEMINI_MODEL");

    let client = Client::new(api_key, model);
    let routes = all_routes();

    let trace = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(
        listener,
        routes
            .layer(allowed_origins)
            .layer(Extension(client))
            .layer(trace),
    )
    .await
    .expect("Server failed to start");
}

fn all_routes() -> Router {
    Router::new()
        .route("/gems/generate-content", post(generate_content))
        .route(
            "/gems/stream-generate-content",
            post(stream_generate_content),
        )
        .route("/gems/count-tokens", post(count_tokens))
        .route("/gems/embed-content", post(embed_content))
        .route("/", post(|| async { StatusCode::NOT_FOUND }))
}

#[derive(Debug, serde::Deserialize)]
struct GenerateContentRequest {
    input_text: String,
}

async fn generate_content(
    Extension(mut client): Extension<Client>,
    Json(request): Json<GenerateContentRequest>,
) -> impl IntoResponse {
    match client.generate_content(&request.input_text).await {
        Ok(response) => response,
        Err(error) => error.to_string(),
    }
}

#[derive(Debug, serde::Deserialize)]
struct StreamGenerateContentRequest {
    input_text: String,
}

async fn stream_generate_content(
    Extension(mut client): Extension<Client>,
    Json(request): Json<StreamGenerateContentRequest>,
) -> impl IntoResponse {
    match client
        .stream_generate_content(&request.input_text, true)
        .await
    {
        Ok(response) => response,
        Err(error) => error.to_string(),
    }
}

#[derive(Debug, serde::Deserialize)]
struct CountTokensRequest {
    input_text: String,
}

async fn count_tokens(
    Extension(mut client): Extension<Client>,
    Json(request): Json<CountTokensRequest>,
) -> impl IntoResponse {
    match client.count_tokens(&request.input_text).await {
        Ok(response) => response.to_string(),
        Err(error) => error.to_string(),
    }
}

#[derive(Debug, serde::Deserialize)]
struct EmbedContentRequest {
    input_text: String,
}

async fn embed_content(
    Extension(client): Extension<Client>,
    Json(request): Json<EmbedContentRequest>,
) -> impl IntoResponse {
    let mut client = Client::new(&client.api_key, "embedding-001");
    let response = client.embed_content(&request.input_text).await.unwrap();
    Json(response.embedding.unwrap().values)
}


Your Output:
[
    {
        "route": "/gems/generate-content",
        "dynamic": "false",
        "method": "post",
        "body": {
            "input_text": "string"
        },
        "response": "string"
    },
    {
        "route": "/gems/stream-generate-content",
        "dynamic": "false",
        "method": "post",
        "body": {
            "input_text": "string"
        },
        "response": "string"
    },
    {
        "route": "/gems/count-tokens",
        "dynamic": "false",
        "method": "post",
        "body": {
            "input_text": "string"
        },
        "response": "string"
    },
    {
        "route": "/gems/embed-content",
        "dynamic": "false",
        "method": "post",
        "body": {
            "input_text": "string"
        },
        "response": "string"
    },
    {
        "route": "/",
        "dynamic": "false",
        "method": "post",
        "body": {},
        "response": "string"
    }
]

User Request:

from fastapi import FastAPI, Request, HTTPException
import requests

app = FastAPI()

@app.get("/")
async def root(request: Request):
    return {"message": "Hello World"}

@app.get("/weather")
async def get_weather(request: Request):
    city = request.query_params.get("city")
    if not city:
        raise HTTPException(status_code=400, detail="Missing city parameter")

    r = requests.get(f"http://api.openweathermap.org/data/2.5/weather?q={city}&appid=YOUR_API_KEY")
    if r.status_code != 200:
        raise HTTPException(status_code=r.status_code, detail=r.text)

    return r.json()


Your Output:
"#;
