use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use crate::config::get_db_path;
use crate::db::get_connection;

/// MCP Server implementation for AgentMem
/// Communicates via JSON-RPC over stdin/stdout

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcRequest {
    jsonrpc: String,  // Required by JSON-RPC spec, validated by serde
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// Run the MCP server - reads JSON-RPC from stdin, writes to stdout
pub fn run_server() -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => {
                let response = handle_request(request);
                let response_json = serde_json::to_string(&response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
            Err(e) => {
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                let response_json = serde_json::to_string(&error_response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
        }
    }

    Ok(())
}

fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.unwrap_or(Value::Null);

    match request.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "agentmem",
                    "version": "2.0.0"
                }
            })),
            error: None,
        },

        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "tools": [
                    {
                        "name": "add_memory",
                        "description": "Add a memory to AgentMem. Types: decision, correction, gotcha, pattern, infrastructure, tool, protected, insight",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "type": { "type": "string", "description": "Memory type" },
                                "title": { "type": "string", "description": "Brief title" },
                                "content": { "type": "string", "description": "Detailed content" }
                            },
                            "required": ["type", "title"]
                        }
                    },
                    {
                        "name": "get_context",
                        "description": "Get relevant context from AgentMem for a query",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": { "type": "string", "description": "Search query" }
                            }
                        }
                    },
                    {
                        "name": "list_tasks",
                        "description": "List all tasks",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "status": { "type": "string", "description": "Filter by status (open, in_progress, closed)" }
                            }
                        }
                    },
                    {
                        "name": "create_task",
                        "description": "Create a new task",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "title": { "type": "string", "description": "Task title" },
                                "description": { "type": "string", "description": "Task description" },
                                "priority": { "type": "integer", "description": "Priority 0-4" },
                                "type": { "type": "string", "description": "Task type (bug, feature, task, etc.)" }
                            },
                            "required": ["title"]
                        }
                    },
                    {
                        "name": "update_task",
                        "description": "Update task status",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "id": { "type": "string", "description": "Task ID" },
                                "status": { "type": "string", "description": "New status (open, in_progress, closed)" }
                            },
                            "required": ["id", "status"]
                        }
                    },
                    {
                        "name": "protect_file",
                        "description": "Mark a file as protected (requires approval to modify)",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "path": { "type": "string", "description": "File path or glob pattern" },
                                "reason": { "type": "string", "description": "Why this file is protected" }
                            },
                            "required": ["path"]
                        }
                    },
                    {
                        "name": "list_protected_files",
                        "description": "List all protected files",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "save_plan",
                        "description": "Save a plan from plan mode",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "title": { "type": "string", "description": "Plan title" },
                                "content": { "type": "string", "description": "Plan content" },
                                "file_path": { "type": "string", "description": "Path to plan file" }
                            },
                            "required": ["title"]
                        }
                    },
                    {
                        "name": "save_todos",
                        "description": "Save TodoWrite state for persistence",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "snapshot": { "type": "string", "description": "JSON representation of TodoWrite state" }
                            },
                            "required": ["snapshot"]
                        }
                    },
                    {
                        "name": "get_todos",
                        "description": "Get the last saved TodoWrite state",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    }
                ]
            })),
            error: None,
        },

        "tools/call" => {
            let params = request.params.unwrap_or(Value::Null);
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            match call_tool(tool_name, arguments) {
                Ok(result) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({
                        "content": [{
                            "type": "text",
                            "text": result
                        }]
                    })),
                    error: None,
                },
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Error: {}", e)
                        }],
                        "isError": true
                    })),
                    error: None,
                },
            }
        }

        "notifications/initialized" => {
            // Notification, no response needed but we'll acknowledge
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(Value::Null),
                error: None,
            }
        }

        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        },
    }
}

fn call_tool(name: &str, args: Value) -> Result<String> {
    let db_path = get_db_path();
    if !db_path.exists() {
        anyhow::bail!("AgentMem not initialized. Run 'am init' first.");
    }
    let conn = get_connection(db_path)?;

    match name {
        "add_memory" => {
            let memory_type = args.get("type").and_then(|v| v.as_str()).unwrap_or("insight");
            let title = args.get("title").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("title required"))?;
            let content = args.get("content").and_then(|v| v.as_str());

            let id = crate::memory::service::add_memory(&conn, memory_type, title, content)?;
            Ok(format!("Added memory: {} \"{}\"", id, title))
        }

        "get_context" => {
            let query = args.get("query").and_then(|v| v.as_str());
            let context = crate::retrieval::context::get_context(&conn, query, None, None, 5, 3)?;
            let markdown = crate::retrieval::context::format_context_markdown(&context);
            Ok(markdown)
        }

        "list_tasks" => {
            let tasks = crate::tasks::service::list_tasks(&conn)?;
            let status_filter = args.get("status").and_then(|v| v.as_str());

            let filtered: Vec<_> = if let Some(status) = status_filter {
                tasks.into_iter().filter(|t| t.status == status).collect()
            } else {
                tasks
            };

            let output: Vec<String> = filtered.iter()
                .map(|t| format!("[P{}] {}: {} ({})", t.priority, t.id, t.title, t.status))
                .collect();

            Ok(output.join("\n"))
        }

        "create_task" => {
            let title = args.get("title").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("title required"))?;
            let description = args.get("description").and_then(|v| v.as_str());
            let priority = args.get("priority").and_then(|v| v.as_i64()).unwrap_or(2) as i32;
            let task_type = args.get("type").and_then(|v| v.as_str()).unwrap_or("task");

            let id = crate::tasks::service::create_task(&conn, title, description, priority, task_type)?;
            Ok(format!("Created task: {} \"{}\"", id, title))
        }

        "update_task" => {
            let id = args.get("id").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("id required"))?;
            let status = args.get("status").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("status required"))?;

            crate::tasks::service::update_task_status(&conn, id, status, "agent", None)?;
            Ok(format!("Updated task {} to: {}", id, status))
        }

        "protect_file" => {
            let path = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("path required"))?;
            let reason = args.get("reason").and_then(|v| v.as_str());

            crate::memory::service::add_protected_file(&conn, path, reason)?;
            Ok(format!("Protected: {}", path))
        }

        "list_protected_files" => {
            let files = crate::memory::service::list_protected_files(&conn)?;
            let output: Vec<String> = files.iter()
                .map(|f| format!("{} - {}", f.pattern, f.reason.as_deref().unwrap_or("no reason")))
                .collect();
            Ok(output.join("\n"))
        }

        "save_plan" => {
            let title = args.get("title").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("title required"))?;
            let content = args.get("content").and_then(|v| v.as_str());
            let file_path = args.get("file_path").and_then(|v| v.as_str());

            let id = crate::plans::service::create_plan(&conn, title, content, file_path)?;
            Ok(format!("Saved plan: {} \"{}\"", id, title))
        }

        "save_todos" => {
            let snapshot = args.get("snapshot").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("snapshot required"))?;

            // Get or create active session
            let session_id = match crate::sessions::service::get_active_session(&conn)? {
                Some(s) => s.id,
                None => crate::sessions::service::start_session(&conn, Some("claude-code"), None)?,
            };

            let snap_id = crate::sessions::service::save_todowrite_snapshot(&conn, &session_id, snapshot)?;
            Ok(format!("Saved TodoWrite snapshot: {}", snap_id))
        }

        "get_todos" => {
            match crate::sessions::service::get_most_recent_snapshot(&conn)? {
                Some(snapshot) => Ok(snapshot.snapshot_json),
                None => Ok("No TodoWrite snapshot found.".to_string()),
            }
        }

        _ => anyhow::bail!("Unknown tool: {}", name),
    }
}
