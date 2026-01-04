use rusqlite::{params, Connection, Result};
use crate::db::models::{Session, TodoWriteSnapshot};
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};

/// Start a new session
pub fn start_session(conn: &Connection, agent: Option<&str>, model: Option<&str>) -> Result<String> {
    let suffix: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let id = format!("sess-{}", suffix.to_lowercase());
    let now = Utc::now();

    conn.execute(
        "INSERT INTO sessions (id, started_at, status, agent, model)
         VALUES (?1, ?2, 'active', ?3, ?4)",
        params![id, now, agent, model],
    )?;

    Ok(id)
}

/// Get a session by ID
pub fn get_session(conn: &Connection, id: &str) -> Result<Option<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, started_at, ended_at, status, agent, model, tokens_in, tokens_out, last_task_id, summary
         FROM sessions WHERE id = ?1"
    )?;

    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Session {
            id: row.get(0)?,
            started_at: row.get(1)?,
            ended_at: row.get(2)?,
            status: row.get(3)?,
            agent: row.get(4)?,
            model: row.get(5)?,
            tokens_in: row.get(6)?,
            tokens_out: row.get(7)?,
            last_task_id: row.get(8)?,
            summary: row.get(9)?,
        }))
    } else {
        Ok(None)
    }
}

/// Get the current active session
pub fn get_active_session(conn: &Connection) -> Result<Option<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, started_at, ended_at, status, agent, model, tokens_in, tokens_out, last_task_id, summary
         FROM sessions WHERE status = 'active' ORDER BY started_at DESC LIMIT 1"
    )?;

    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Session {
            id: row.get(0)?,
            started_at: row.get(1)?,
            ended_at: row.get(2)?,
            status: row.get(3)?,
            agent: row.get(4)?,
            model: row.get(5)?,
            tokens_in: row.get(6)?,
            tokens_out: row.get(7)?,
            last_task_id: row.get(8)?,
            summary: row.get(9)?,
        }))
    } else {
        Ok(None)
    }
}

/// End a session
pub fn end_session(conn: &Connection, id: &str, summary: Option<&str>) -> Result<()> {
    let now = Utc::now();
    conn.execute(
        "UPDATE sessions SET ended_at = ?1, status = 'completed', summary = ?2 WHERE id = ?3",
        params![now, summary, id],
    )?;
    Ok(())
}

/// Update session token counts
pub fn update_session_tokens(conn: &Connection, id: &str, tokens_in: i32, tokens_out: i32) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET tokens_in = tokens_in + ?1, tokens_out = tokens_out + ?2 WHERE id = ?3",
        params![tokens_in, tokens_out, id],
    )?;
    Ok(())
}

/// Update last task worked on
pub fn update_last_task(conn: &Connection, session_id: &str, task_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET last_task_id = ?1 WHERE id = ?2",
        params![task_id, session_id],
    )?;
    Ok(())
}

/// List recent sessions
pub fn list_sessions(conn: &Connection, limit: i32) -> Result<Vec<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, started_at, ended_at, status, agent, model, tokens_in, tokens_out, last_task_id, summary
         FROM sessions ORDER BY started_at DESC LIMIT ?1"
    )?;

    let rows = stmt.query_map(params![limit], |row| {
        Ok(Session {
            id: row.get(0)?,
            started_at: row.get(1)?,
            ended_at: row.get(2)?,
            status: row.get(3)?,
            agent: row.get(4)?,
            model: row.get(5)?,
            tokens_in: row.get(6)?,
            tokens_out: row.get(7)?,
            last_task_id: row.get(8)?,
            summary: row.get(9)?,
        })
    })?;

    let mut sessions = Vec::new();
    for session in rows {
        sessions.push(session?);
    }
    Ok(sessions)
}

/// Save a TodoWrite snapshot
pub fn save_todowrite_snapshot(conn: &Connection, session_id: &str, snapshot_json: &str) -> Result<String> {
    let suffix: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();
    let id = format!("snap-{}", suffix.to_lowercase());
    let now = Utc::now();

    conn.execute(
        "INSERT INTO todowrite_snapshots (id, session_id, snapshot_json, captured_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![id, session_id, snapshot_json, now],
    )?;

    Ok(id)
}

/// Get the latest TodoWrite snapshot for a session
pub fn get_latest_snapshot(conn: &Connection, session_id: &str) -> Result<Option<TodoWriteSnapshot>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, snapshot_json, captured_at
         FROM todowrite_snapshots WHERE session_id = ?1 ORDER BY captured_at DESC LIMIT 1"
    )?;

    let mut rows = stmt.query(params![session_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(TodoWriteSnapshot {
            id: row.get(0)?,
            session_id: row.get(1)?,
            snapshot_json: row.get(2)?,
            captured_at: row.get(3)?,
        }))
    } else {
        Ok(None)
    }
}

/// Get the most recent snapshot across all sessions
pub fn get_most_recent_snapshot(conn: &Connection) -> Result<Option<TodoWriteSnapshot>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, snapshot_json, captured_at
         FROM todowrite_snapshots ORDER BY captured_at DESC LIMIT 1"
    )?;

    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(Some(TodoWriteSnapshot {
            id: row.get(0)?,
            session_id: row.get(1)?,
            snapshot_json: row.get(2)?,
            captured_at: row.get(3)?,
        }))
    } else {
        Ok(None)
    }
}
