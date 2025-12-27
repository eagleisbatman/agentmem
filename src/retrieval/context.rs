use rusqlite::{Connection, Result};
use crate::db::models::{Task, Memory, ProtectedFile, Tool};
use crate::tasks::service::get_ready_tasks;
use crate::memory::service::list_memories;
use crate::retrieval::search::{semantic_search, is_semantic_search_available};
use serde::{Deserialize, Serialize};
use anyhow::Result as AnyhowResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextResponse {
    pub tasks: Vec<Task>,
    pub memories: Vec<Memory>,
    pub protected: Vec<ProtectedFile>,
    pub tools: Vec<Tool>,
}

pub fn get_context(
    conn: &Connection,
    query: Option<&str>,
    _task_id: Option<&str>,
    _file_path: Option<&str>,
    limit_memories: usize,
    limit_tasks: usize,
) -> Result<ContextResponse> {
    // 1. Get Tasks (unblocked/ready)
    let mut tasks = get_ready_tasks(conn)?;
    tasks.truncate(limit_tasks);

    // 2. Get Memories (for now, just a simple keyword search if query is provided, or top N)
    let memories = if let Some(q) = query {
        let mut stmt = conn.prepare("SELECT id, type, title, content, source_chunk, confidence, times_recalled, first_observed_at, last_observed_at, last_recalled_at, created_at, updated_at FROM memories WHERE title LIKE ?1 OR content LIKE ?1 LIMIT ?2")?;
        let search_term = format!("%{}%", q);
        let memory_iter = stmt.query_map([search_term, limit_memories.to_string()], |row| {
            use uuid::Uuid;
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str).unwrap_or_default();
            Ok(Memory {
                id,
                memory_type: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                source_chunk: row.get(4)?,
                confidence: row.get(5)?,
                times_recalled: row.get(6)?,
                first_observed_at: row.get(7)?,
                last_observed_at: row.get(8)?,
                last_recalled_at: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?;
        let mut results = Vec::new();
        for m in memory_iter {
            results.push(m?);
        }
        results
    } else {
        let mut memories = list_memories(conn)?;
        memories.truncate(limit_memories);
        memories
    };

    // 3. Get Protected Files (always include)
    let mut stmt = conn.prepare("SELECT pattern, reason, added_at FROM protected_files")?;
    let protected_iter = stmt.query_map([], |row| {
        Ok(ProtectedFile {
            pattern: row.get(0)?,
            reason: row.get(1)?,
            added_at: row.get(2)?,
        })
    })?;
    let mut protected = Vec::new();
    for p in protected_iter {
        protected.push(p?);
    }

    // 4. Get Tools
    let mut stmt = conn.prepare("SELECT id, name, location, description, usage, added_at FROM tools")?;
    let tool_iter = stmt.query_map([], |row| {
        Ok(Tool {
            id: row.get(0)?,
            name: row.get(1)?,
            location: row.get(2)?,
            description: row.get(3)?,
            usage: row.get(4)?,
            added_at: row.get(5)?,
        })
    })?;
    let mut tools = Vec::new();
    for t in tool_iter {
        tools.push(t?);
    }

    Ok(ContextResponse {
        tasks,
        memories,
        protected,
        tools,
    })
}

/// Get context with semantic search (async version)
/// Falls back to LIKE search if semantic search is unavailable
pub async fn get_context_async(
    conn: &Connection,
    query: Option<&str>,
    _task_id: Option<&str>,
    _file_path: Option<&str>,
    limit_memories: usize,
    limit_tasks: usize,
) -> AnyhowResult<ContextResponse> {
    // 1. Get Tasks (unblocked/ready)
    let mut tasks = get_ready_tasks(conn)
        .map_err(|e| anyhow::anyhow!("Failed to get tasks: {}", e))?;
    tasks.truncate(limit_tasks);

    // 2. Get Memories - try semantic search first if query provided
    let memories = if let Some(q) = query {
        // Try semantic search
        match semantic_search(conn, q, limit_memories).await {
            Ok(results) => {
                // Extract memories from search results
                results.into_iter().map(|r| r.memory).collect()
            }
            Err(_) => {
                // Fall back to LIKE search
                fallback_memory_search(conn, q, limit_memories)?
            }
        }
    } else {
        let mut memories = list_memories(conn)
            .map_err(|e| anyhow::anyhow!("Failed to list memories: {}", e))?;
        memories.truncate(limit_memories);
        memories
    };

    // 3. Get Protected Files (always include)
    let protected = get_protected_files(conn)?;

    // 4. Get Tools
    let tools = get_tools(conn)?;

    Ok(ContextResponse {
        tasks,
        memories,
        protected,
        tools,
    })
}

/// Fallback LIKE-based memory search
fn fallback_memory_search(conn: &Connection, query: &str, limit: usize) -> AnyhowResult<Vec<Memory>> {
    let mut stmt = conn.prepare(
        "SELECT id, type, title, content, source_chunk, confidence, times_recalled,
         first_observed_at, last_observed_at, last_recalled_at, created_at, updated_at
         FROM memories WHERE title LIKE ?1 OR content LIKE ?1 LIMIT ?2"
    )?;

    let search_term = format!("%{}%", query);
    let memory_iter = stmt.query_map([search_term, limit.to_string()], |row| {
        use uuid::Uuid;
        let id_str: String = row.get(0)?;
        let id = Uuid::parse_str(&id_str).unwrap_or_default();
        Ok(Memory {
            id,
            memory_type: row.get(1)?,
            title: row.get(2)?,
            content: row.get(3)?,
            source_chunk: row.get(4)?,
            confidence: row.get(5)?,
            times_recalled: row.get(6)?,
            first_observed_at: row.get(7)?,
            last_observed_at: row.get(8)?,
            last_recalled_at: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })?;

    let mut results = Vec::new();
    for m in memory_iter {
        results.push(m?);
    }
    Ok(results)
}

/// Get protected files from database
fn get_protected_files(conn: &Connection) -> AnyhowResult<Vec<ProtectedFile>> {
    let mut stmt = conn.prepare("SELECT pattern, reason, added_at FROM protected_files")?;
    let iter = stmt.query_map([], |row| {
        Ok(ProtectedFile {
            pattern: row.get(0)?,
            reason: row.get(1)?,
            added_at: row.get(2)?,
        })
    })?;

    let mut results = Vec::new();
    for item in iter {
        results.push(item?);
    }
    Ok(results)
}

/// Get tools from database
fn get_tools(conn: &Connection) -> AnyhowResult<Vec<Tool>> {
    let mut stmt = conn.prepare("SELECT id, name, location, description, usage, added_at FROM tools")?;
    let iter = stmt.query_map([], |row| {
        Ok(Tool {
            id: row.get(0)?,
            name: row.get(1)?,
            location: row.get(2)?,
            description: row.get(3)?,
            usage: row.get(4)?,
            added_at: row.get(5)?,
        })
    })?;

    let mut results = Vec::new();
    for item in iter {
        results.push(item?);
    }
    Ok(results)
}

pub fn format_context_markdown(context: &ContextResponse) -> String {
    let mut output = String::new();

    if !context.protected.is_empty() {
        output.push_str("## ⚠️ Protected Files\n");
        output.push_str("Ask before modifying:\n");
        for p in &context.protected {
            output.push_str(&format!("- `{}` — {}\n", p.pattern, p.reason.as_deref().unwrap_or("No reason provided")));
        }
        output.push_str("\n");
    }

    if !context.tasks.is_empty() {
        output.push_str("## Current Tasks\n");
        for t in &context.tasks {
            output.push_str(&format!("- [P{}] {}: {} ({})\n", t.priority, t.id, t.title, t.status));
        }
        output.push_str("\n");
    }

    if !context.memories.is_empty() {
        output.push_str("## Relevant Context\n");
        for m in &context.memories {
            output.push_str(&format!("- [{}] {}: {}\n", m.memory_type, m.title, m.content.as_deref().unwrap_or("")));
        }
        output.push_str("\n");
    }

    if !context.tools.is_empty() {
        output.push_str("## Available Tools\n");
        for t in &context.tools {
            output.push_str(&format!("- `{}` — {}\n", t.location, t.description.as_deref().unwrap_or("")));
        }
    }

    output
}

