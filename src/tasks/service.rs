use rusqlite::{params, Connection, Result};
use crate::db::models::{Task, TaskHistory};
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};

pub fn create_task(conn: &Connection, title: &str, description: Option<&str>, priority: i32, task_type: &str) -> Result<String> {
    let suffix: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(4)
        .map(char::from)
        .collect();
    let id = format!("am-{}", suffix.to_lowercase());
    let now = Utc::now();

    conn.execute(
        "INSERT INTO tasks (id, title, description, priority, type, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, title, description, priority, task_type, now, now],
    )?;

    Ok(id)
}

pub fn list_tasks(conn: &Connection) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare("SELECT id, title, description, status, priority, type, labels, assignee, notes, created_at, updated_at, closed_at, closed_reason FROM tasks")?;
    let task_iter = stmt.query_map([], |row| {
        let labels_str: Option<String> = row.get(6)?;
        let labels = if let Some(s) = labels_str {
            serde_json::from_str(&s).unwrap_or_default()
        } else {
            vec![]
        };

        Ok(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            status: row.get(3)?,
            priority: row.get(4)?,
            task_type: row.get(5)?,
            labels,
            assignee: row.get(7)?,
            notes: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
            closed_at: row.get(11)?,
            closed_reason: row.get(12)?,
        })
    })?;

    let mut tasks = Vec::new();
    for task in task_iter {
        tasks.push(task?);
    }
    Ok(tasks)
}

pub fn get_ready_tasks(conn: &Connection) -> Result<Vec<Task>> {
    // Basic implementation: open tasks that are not blocked
    // For now, just return all open tasks since dependency logic is not yet implemented
    let mut stmt = conn.prepare("SELECT id, title, description, status, priority, type, labels, assignee, notes, created_at, updated_at, closed_at, closed_reason FROM tasks WHERE status = 'open' ORDER BY priority ASC")?;
    let task_iter = stmt.query_map([], |row| {
        let labels_str: Option<String> = row.get(6)?;
        let labels = if let Some(s) = labels_str {
            serde_json::from_str(&s).unwrap_or_default()
        } else {
            vec![]
        };

        Ok(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            status: row.get(3)?,
            priority: row.get(4)?,
            task_type: row.get(5)?,
            labels,
            assignee: row.get(7)?,
            notes: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
            closed_at: row.get(11)?,
            closed_reason: row.get(12)?,
        })
    })?;

    let mut tasks = Vec::new();
    for task in task_iter {
        tasks.push(task?);
    }
    Ok(tasks)
}

/// Get a task by ID
pub fn get_task(conn: &Connection, task_id: &str) -> Result<Option<Task>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, status, priority, type, labels, assignee, notes, created_at, updated_at, closed_at, closed_reason
         FROM tasks WHERE id = ?1"
    )?;

    let mut rows = stmt.query(params![task_id])?;
    if let Some(row) = rows.next()? {
        let labels_str: Option<String> = row.get(6)?;
        let labels = if let Some(s) = labels_str {
            serde_json::from_str(&s).unwrap_or_default()
        } else {
            vec![]
        };

        Ok(Some(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            status: row.get(3)?,
            priority: row.get(4)?,
            task_type: row.get(5)?,
            labels,
            assignee: row.get(7)?,
            notes: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
            closed_at: row.get(11)?,
            closed_reason: row.get(12)?,
        }))
    } else {
        Ok(None)
    }
}

/// Update a task's status and record history
/// Returns an error if the task doesn't exist
pub fn update_task_status(conn: &Connection, task_id: &str, new_status: &str, changed_by: &str, notes: Option<&str>) -> Result<()> {
    let now = Utc::now();

    // Get current status - fail if task doesn't exist
    let old_status: String = conn.query_row(
        "SELECT status FROM tasks WHERE id = ?1",
        params![task_id],
        |row| row.get(0)
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            rusqlite::Error::QueryReturnedNoRows
        }
        other => other
    })?;

    // Update the task
    let rows_affected = if new_status == "closed" {
        conn.execute(
            "UPDATE tasks SET status = ?1, updated_at = ?2, closed_at = ?2 WHERE id = ?3",
            params![new_status, now, task_id],
        )?
    } else {
        conn.execute(
            "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![new_status, now, task_id],
        )?
    };

    // Double-check rows were affected (shouldn't fail after SELECT succeeded, but be safe)
    if rows_affected == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    // Record history
    record_task_history(conn, task_id, Some(&old_status), new_status, changed_by, notes)?;

    Ok(())
}

/// Record a task status change in history
pub fn record_task_history(conn: &Connection, task_id: &str, old_status: Option<&str>, new_status: &str, changed_by: &str, notes: Option<&str>) -> Result<String> {
    let suffix: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();
    let id = format!("hist-{}", suffix.to_lowercase());
    let now = Utc::now();

    conn.execute(
        "INSERT INTO task_history (id, task_id, old_status, new_status, changed_at, changed_by, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, task_id, old_status, new_status, now, changed_by, notes],
    )?;

    Ok(id)
}

/// Get history for a task
pub fn get_task_history(conn: &Connection, task_id: &str) -> Result<Vec<TaskHistory>> {
    let mut stmt = conn.prepare(
        "SELECT id, task_id, old_status, new_status, changed_at, changed_by, notes
         FROM task_history WHERE task_id = ?1 ORDER BY changed_at DESC"
    )?;

    let rows = stmt.query_map(params![task_id], |row| {
        Ok(TaskHistory {
            id: row.get(0)?,
            task_id: row.get(1)?,
            old_status: row.get(2)?,
            new_status: row.get(3)?,
            changed_at: row.get(4)?,
            changed_by: row.get(5)?,
            notes: row.get(6)?,
        })
    })?;

    let mut history = Vec::new();
    for h in rows {
        history.push(h?);
    }
    Ok(history)
}
