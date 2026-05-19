# MCP SDK Usage

In addition to the CLI and TUI, you can interact with MCP programmatically from Rust using the `McpClient`, `McpManager`, and settings types directly.

## Feature Flag

```toml
# Cargo.toml
[dependencies]
autogpt = { version = "0.4.4", features = ["mcp"] }
```

## Core Types

| Type              | Location                 | Purpose                                 |
| ----------------- | ------------------------ | --------------------------------------- |
| `McpServerConfig` | `autogpt::mcp::settings` | Single server configuration             |
| `McpTransport`    | `autogpt::mcp::settings` | Transport enum: Stdio, Http, Sse        |
| `McpClient`       | `autogpt::mcp::client`   | Low-level client for one server         |
| `McpManager`      | `autogpt::mcp::manager`  | High-level manager for multiple servers |
| `McpTool`         | `autogpt::mcp::types`    | Discovered tool descriptor              |
| `McpServerInfo`   | `autogpt::mcp::types`    | Connection status + tool list           |
| `SettingsManager` | `autogpt::cli::settings` | Read/write `~/.autogpt/settings.json`   |

## `McpClient`: Connecting to a Single Server

`McpClient` manages the lifecycle for one MCP server: connect, discover tools, call tools.

### Stdio server

```rust
use autogpt::prelude::*;
use std::collections::HashMap;

fn main() -> Result<()> {
    let config = McpServerConfig {
        name: "everything".into(),
        transport: McpTransport::Stdio,
        command: Some("npx".into()),
        args: vec!["-y".into(), "@modelcontextprotocol/server-everything".into()],
        env: HashMap::new(),
        headers: HashMap::new(),
        timeout_ms: 30_000,
        trust: true,
        ..Default::default()
    };

    let mut client = McpClient::new("everything");
    client.connect(&config)?;

    let info = client.to_server_info("Everything test server");
    println!("Status: {:?}", info.status);
    for tool in &info.tools {
        println!("  {} - {}", tool.name, tool.description);
    }

    let mut args = HashMap::new();
    args.insert("message".into(), serde_json::json!("hello"));
    let result = client.call_tool("echo", args)?;
    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

// Status: Connected
//   echo - Echoes back the input string
//   get-annotated-message - Demonstrates how annotations can be used to provide metadata about content.
//   get-env - Returns all environment variables, helpful for debugging MCP server configuration
//   get-resource-links - Returns up to ten resource links that reference different types of resources
//   get-resource-reference - Returns a resource reference that can be used by MCP clients
//   get-structured-content - Returns structured content along with an output schema for client data validation
//   get-sum - Returns the sum of two numbers
//   get-tiny-image - Returns a tiny MCP logo image.
//   gzip-file-as-resource - Compresses a single file using gzip compression. Depending upon the selected output type, returns either the compressed data as a gzipped resource or a resource link, allowing it to be downloaded in a subsequent request during the current session.
//   toggle-simulated-logging - Toggles simulated, random-leveled logging on or off.
//   toggle-subscriber-updates - Toggles simulated resource subscription updates on or off.
//   trigger-long-running-operation - Demonstrates a long running operation with progress updates.
//   simulate-research-query - Simulates a deep research operation that gathers, analyzes, and synthesizes information. Demonstrates MCP task-based operations with progress through multiple stages. If 'ambiguous' is true and client supports elicitation, sends an elicitation request for clarification.
// {
//   "success": true,
//   "content": "Echo: hello",
//   "data": null,
//   "error": null
// }
```

### HTTP server with auth header

```rust
use autogpt::prelude::*;
use std::collections::HashMap;

let mut headers = HashMap::new();
headers.insert(
    "Authorization".into(),
    "Bearer my-secret-token".into(), // or "$MY_TOKEN" for expansion
);

let config = McpServerConfig {
    name: "secure-api".into(),
    transport: McpTransport::Http,
    http_url: Some("https://api.wiseai.dev/mcp".into()),
    headers,
    timeout_ms: 10_000,
    ..Default::default()
};
```

### Filtering tools

```rust
let config = McpServerConfig {
    name: "filtered".into(),
    transport: McpTransport::Stdio,
    command: Some("npx".into()),
    args: vec!["-y".into(), "@modelcontextprotocol/server-everything".into()],
    include_tools: vec!["echo".into(), "add".into()],
    exclude_tools: vec!["get-env".into()],
    ..Default::default()
};
```

## `McpManager`: Managing Multiple Servers

`McpManager` connects to all configured servers at once and provides a unified tool dispatch interface. This is what agents use internally.

```rust
use autogpt::prelude::*;
use std::collections::HashMap;

let mut servers: HashMap<String, McpServerConfig> = HashMap::new();
servers.insert("everything".into(), McpServerConfig {
    name: "everything".into(),
    transport: autogpt::mcp::settings::McpTransport::Stdio,
    command: Some("npx".into()),
    args: vec!["-y".into(), "@modelcontextprotocol/server-everything".into()],
    trust: true,
    ..Default::default()
});

let mut mgr = McpManager::new(servers, vec![], vec![]);
mgr.connect_all();

for info in mgr.server_infos() {
    println!("Server: {}", info.name);
    for tool in info.tools {
        println!("  {}", tool.fqn); // e.g. "mcp_everything_echo"
    }
}

let mut args = HashMap::new();
args.insert("message".into(), serde_json::json!("hello from sdk"));
let result = mgr.call_tool("mcp_everything_echo", args).await?;
println!("{}", serde_json::to_string_pretty(&result)?);
```

## Tool FQN Naming Convention

Every discovered tool is assigned a **Fully Qualified Name** (FQN) to avoid collisions across servers:

```
mcp_{server_name}_{tool_name}
```

Non-alphanumeric characters (except `-`) are replaced with `_`. Examples:

| Server name  | Tool name      | FQN                       |
| ------------ | -------------- | ------------------------- |
| `everything` | `echo`         | `mcp_everything_echo`     |
| `my-api`     | `query-data`   | `mcp_my-api_query-data`   |
| `github`     | `create_issue` | `mcp_github_create_issue` |

## `SettingsManager`: Read/Write Config Programmatically

```rust
use autogpt::prelude::*;

// Load from default path (~/.autogpt/settings.json)
let mgr = SettingsManager::new();

let config = McpServerConfig {
    name: "my-server".into(),
    transport: McpTransport::Stdio,
    command: Some("npx".into()),
    args: vec!["-y".into(), "@modelcontextprotocol/server-everything".into()],
    description: Some("Test server".into()),
    ..Default::default()
};

mgr.add_mcp_server(config)?;

let settings = mgr.load()?;
for (name, cfg) in &settings.mcp {
    println!("{}: {:?}", name, cfg.transport);
}

let (_, existed) = mgr.remove_mcp_server("my-server")?;
assert!(existed);
```

## Examples

Full working examples are available in the repository:

| Example                          | Description                            |
| -------------------------------- | -------------------------------------- |
| [`mcp-server-registration`][ex1] | Register MCP servers on an agent       |
| [`mcp-manager`][ex2]             | Direct use of `McpManager`             |
| [`mcp-custom-agent`][ex3]        | Custom agent with pre-loaded MCP tools |

[ex1]: https://github.com/wiseaidotdev/autogpt/tree/main/examples/mcp-server-registration
[ex2]: https://github.com/wiseaidotdev/autogpt/tree/main/examples/mcp-manager
[ex3]: https://github.com/wiseaidotdev/autogpt/tree/main/examples/mcp-custom-agent
