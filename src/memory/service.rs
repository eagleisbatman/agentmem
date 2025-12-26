use rusqlite::{params, Connection, Result};
use crate::db::models::Memory;
use chrono::Utc;
use uuid::Uuid;

pub fn add_memory(
    conn: &Connection,
    memory_type: &str,
    title: &str,
    content: Option<&str>,
) -> Result<Uuid> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    conn.execute(
        "INSERT INTO memories (id, type, title, content, first_observed_at, last_observed_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id.to_string(), memory_type, title, content, now, now, now, now],
    )?;

    Ok(id)
}

pub fn list_memories(conn: &Connection) -> Result<Vec<Memory>> {
    let mut stmt = conn.prepare("SELECT id, type, title, content, source_chunk, confidence, times_recalled, first_observed_at, last_observed_at, last_recalled_at, created_at, updated_at FROM memories")?;
    let memory_iter = stmt.query_map([], |row| {
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

    let mut memories = Vec::new();
    for memory in memory_iter {
        memories.push(memory?);
    }
    Ok(memories)
}

pub fn add_protected_file(conn: &Connection, pattern: &str, reason: Option<&str>) -> Result<()> {
    let now = Utc::now();
    conn.execute(
        "INSERT INTO protected_files (pattern, reason, added_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(pattern) DO UPDATE SET reason = excluded.reason, added_at = excluded.added_at",
        params![pattern, reason, now],
    )?;
    Ok(())
}

pub fn add_tool(conn: &Connection, location: &str, name: &str, description: Option<&str>, usage: Option<&str>) -> Result<()> {
    let now = Utc::now();
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO tools (id, name, location, description, usage, added_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, name, location, description, usage, now],
    )?;
    Ok(())
}
