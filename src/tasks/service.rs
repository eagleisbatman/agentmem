use rusqlite::{params, Connection, Result};
use crate::db::models::Task;
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

