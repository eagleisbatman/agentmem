use rusqlite::{params, Connection, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use serde_json::Value;

pub fn import_from_jsonl<P: AsRef<Path>>(conn: &Connection, path: P) -> Result<()> {
    if !path.as_ref().exists() {
        return Ok(());
    }

    let file = File::open(path).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        if line.trim().is_empty() {
            continue;
        }

        let val: Value = serde_json::from_str(&line).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        let item_type = val.get("_type").and_then(|v| v.as_str()).unwrap_or("");

        match item_type {
            "task" => {
                let labels = val.get("labels").and_then(|v| Some(v.to_string())).unwrap_or("[]".to_string());
                conn.execute(
                    "INSERT INTO tasks (id, title, description, status, priority, type, labels, assignee, notes, created_at, updated_at, closed_at, closed_reason)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                     ON CONFLICT(id) DO UPDATE SET
                        title = excluded.title,
                        description = excluded.description,
                        status = excluded.status,
                        priority = excluded.priority,
                        type = excluded.type,
                        labels = excluded.labels,
                        assignee = excluded.assignee,
                        notes = excluded.notes,
                        updated_at = excluded.updated_at,
                        closed_at = excluded.closed_at,
                        closed_reason = excluded.closed_reason",
                    params![
                        val.get("id").and_then(|v| v.as_str()),
                        val.get("title").and_then(|v| v.as_str()),
                        val.get("description").and_then(|v| v.as_str()),
                        val.get("status").and_then(|v| v.as_str()),
                        val.get("priority").and_then(|v| v.as_i64()),
                        val.get("task_type").and_then(|v| v.as_str()),
                        labels,
                        val.get("assignee").and_then(|v| v.as_str()),
                        val.get("notes").and_then(|v| v.as_str()),
                        val.get("created_at").and_then(|v| v.as_str()),
                        val.get("updated_at").and_then(|v| v.as_str()),
                        val.get("closed_at").and_then(|v| v.as_str()),
                        val.get("closed_reason").and_then(|v| v.as_str()),
                    ],
                )?;
            },
            "memory" => {
                conn.execute(
                    "INSERT INTO memories (id, type, title, content, source_chunk, confidence, times_recalled, first_observed_at, last_observed_at, last_recalled_at, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                     ON CONFLICT(id) DO UPDATE SET
                        type = excluded.type,
                        title = excluded.title,
                        content = excluded.content,
                        source_chunk = excluded.source_chunk,
                        confidence = excluded.confidence,
                        times_recalled = excluded.times_recalled,
                        last_observed_at = excluded.last_observed_at,
                        last_recalled_at = excluded.last_recalled_at,
                        updated_at = excluded.updated_at",
                    params![
                        val.get("id").and_then(|v| v.as_str()),
                        val.get("memory_type").and_then(|v| v.as_str()),
                        val.get("title").and_then(|v| v.as_str()),
                        val.get("content").and_then(|v| v.as_str()),
                        val.get("source_chunk").and_then(|v| v.as_str()),
                        val.get("confidence").and_then(|v| v.as_i64()),
                        val.get("times_recalled").and_then(|v| v.as_i64()),
                        val.get("first_observed_at").and_then(|v| v.as_str()),
                        val.get("last_observed_at").and_then(|v| v.as_str()),
                        val.get("last_recalled_at").and_then(|v| v.as_str()),
                        val.get("created_at").and_then(|v| v.as_str()),
                        val.get("updated_at").and_then(|v| v.as_str()),
                    ],
                )?;
            },
            "protected" => {
                conn.execute(
                    "INSERT INTO protected_files (pattern, reason, added_at)
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(pattern) DO UPDATE SET
                        reason = excluded.reason,
                        added_at = excluded.added_at",
                    params![
                        val.get("pattern").and_then(|v| v.as_str()),
                        val.get("reason").and_then(|v| v.as_str()),
                        val.get("added_at").and_then(|v| v.as_str()),
                    ],
                )?;
            },
            "tool" => {
                conn.execute(
                    "INSERT INTO tools (id, name, location, description, usage, added_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                     ON CONFLICT(id) DO UPDATE SET
                        name = excluded.name,
                        location = excluded.location,
                        description = excluded.description,
                        usage = excluded.usage,
                        added_at = excluded.added_at",
                    params![
                        val.get("id").and_then(|v| v.as_str()),
                        val.get("name").and_then(|v| v.as_str()),
                        val.get("location").and_then(|v| v.as_str()),
                        val.get("description").and_then(|v| v.as_str()),
                        val.get("usage").and_then(|v| v.as_str()),
                        val.get("added_at").and_then(|v| v.as_str()),
                    ],
                )?;
            },
            _ => {}
        }
    }

    Ok(())
}

