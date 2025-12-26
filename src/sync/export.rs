use rusqlite::{Connection, Result};
use std::fs::File;
use std::io::{Write, BufWriter};
use std::path::Path;
use crate::tasks::service::list_tasks;
use crate::memory::service::list_memories;
use serde_json::json;
use chrono::Utc;

pub fn export_to_jsonl<P: AsRef<Path>>(conn: &Connection, path: P) -> Result<()> {
    let file = File::create(path).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    let mut writer = BufWriter::new(file);

    // Export Tasks
    let tasks = list_tasks(conn)?;
    for task in tasks {
        let mut val = serde_json::to_value(&task).unwrap();
        val.as_object_mut().unwrap().insert("_type".to_string(), json!("task"));
        val.as_object_mut().unwrap().insert("_ts".to_string(), json!(Utc::now()));
        writeln!(writer, "{}", serde_json::to_string(&val).unwrap()).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    }

    // Export Memories
    let memories = list_memories(conn)?;
    for memory in memories {
        let mut val = serde_json::to_value(&memory).unwrap();
        val.as_object_mut().unwrap().insert("_type".to_string(), json!("memory"));
        val.as_object_mut().unwrap().insert("_ts".to_string(), json!(Utc::now()));
        writeln!(writer, "{}", serde_json::to_string(&val).unwrap()).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    }

    // Export Protected Files
    let mut stmt = conn.prepare("SELECT pattern, reason, added_at FROM protected_files")?;
    let rows = stmt.query_map([], |row| {
        Ok(json!({
            "_type": "protected",
            "pattern": row.get::<_, String>(0)?,
            "reason": row.get::<_, Option<String>>(1)?,
            "added_at": row.get::<_, chrono::DateTime<Utc>>(2)?,
            "_ts": Utc::now(),
        }))
    })?;

    for row in rows {
        writeln!(writer, "{}", serde_json::to_string(&row?).unwrap()).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    }

    // Export Tools
    let mut stmt = conn.prepare("SELECT id, name, location, description, usage, added_at FROM tools")?;
    let rows = stmt.query_map([], |row| {
        Ok(json!({
            "_type": "tool",
            "id": row.get::<_, String>(0)?,
            "name": row.get::<_, String>(1)?,
            "location": row.get::<_, String>(2)?,
            "description": row.get::<_, Option<String>>(3)?,
            "usage": row.get::<_, Option<String>>(4)?,
            "added_at": row.get::<_, chrono::DateTime<Utc>>(5)?,
            "_ts": Utc::now(),
        }))
    })?;

    for row in rows {
        writeln!(writer, "{}", serde_json::to_string(&row?).unwrap()).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    }

    writer.flush().map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
    Ok(())
}
