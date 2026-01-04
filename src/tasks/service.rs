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

// ============================================
// Sub-Agent Task Queue Coordination
// ============================================

/// Claim a task for an agent (locks it from other agents)
/// Returns Ok(true) if claimed, Ok(false) if already claimed by another
pub fn claim_task(conn: &Connection, task_id: &str, agent_id: &str) -> Result<bool> {
    let now = Utc::now();

    // Atomic claim - only succeeds if task is unclaimed OR already claimed by this agent
    // This prevents TOCTOU race where two agents both read NULL then both UPDATE
    let rows_affected = conn.execute(
        "UPDATE tasks SET claimed_by = ?1, claimed_at = ?2, status = 'in_progress', updated_at = ?2
         WHERE id = ?3 AND (claimed_by IS NULL OR claimed_by = '' OR claimed_by = ?1)",
        params![agent_id, now, task_id],
    )?;

    // If no rows updated, either task doesn't exist or claimed by another agent
    if rows_affected == 0 {
        // Check if task exists to give better error context
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM tasks WHERE id = ?1)",
            params![task_id],
            |row| row.get(0)
        ).unwrap_or(false);

        if !exists {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        // Task exists but claimed by another agent
        return Ok(false);
    }

    // Record in history
    record_task_history(conn, task_id, Some("open"), "in_progress", agent_id, Some(&format!("Claimed by agent {}", agent_id)))?;

    Ok(true)
}

/// Release a claimed task (makes it available again)
pub fn release_task(conn: &Connection, task_id: &str, agent_id: &str) -> Result<bool> {
    let now = Utc::now();

    // Atomic release - only succeeds if this agent owns the claim
    // This prevents TOCTOU race conditions
    let rows_affected = conn.execute(
        "UPDATE tasks SET claimed_by = NULL, claimed_at = NULL, status = 'open', updated_at = ?1
         WHERE id = ?2 AND claimed_by = ?3",
        params![now, task_id, agent_id],
    )?;

    // If no rows updated, either task doesn't exist, not claimed, or claimed by another
    if rows_affected == 0 {
        // Check current state to determine reason
        let claimed_by: Option<String> = conn.query_row(
            "SELECT claimed_by FROM tasks WHERE id = ?1",
            params![task_id],
            |row| row.get(0)
        ).ok().flatten();

        match claimed_by {
            None => return Ok(true),  // Not claimed, nothing to release (idempotent)
            Some(ref owner) if owner.is_empty() => return Ok(true), // Same as above
            Some(_) => return Ok(false), // Claimed by another agent
        }
    }

    // Record in history
    record_task_history(conn, task_id, Some("in_progress"), "open", agent_id, Some("Released by agent"))?;

    Ok(true)
}

/// Release all tasks claimed by an agent (for session cleanup)
/// Returns the number of tasks released
pub fn release_all_agent_tasks(conn: &Connection, agent_id: &str) -> Result<usize> {
    let now = Utc::now();

    // Get tasks claimed by this agent before releasing (for history)
    let claimed_tasks: Vec<String> = {
        let mut stmt = conn.prepare("SELECT id FROM tasks WHERE claimed_by = ?1")?;
        let rows = stmt.query_map(params![agent_id], |row| row.get(0))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    // Release all at once
    let rows_affected = conn.execute(
        "UPDATE tasks SET claimed_by = NULL, claimed_at = NULL, status = 'open', updated_at = ?1
         WHERE claimed_by = ?2",
        params![now, agent_id],
    )?;

    // Record history for each released task
    for task_id in &claimed_tasks {
        let _ = record_task_history(conn, task_id, Some("in_progress"), "open", agent_id, Some("Released on session end"));
    }

    Ok(rows_affected)
}

/// Release stale claims (tasks claimed more than timeout_minutes ago)
/// Returns the number of tasks released
pub fn release_stale_claims(conn: &Connection, timeout_minutes: i64) -> Result<usize> {
    let now = Utc::now();
    let cutoff = now - chrono::Duration::minutes(timeout_minutes);

    // Get stale tasks before releasing (for history)
    let stale_tasks: Vec<(String, String)> = {
        let mut stmt = conn.prepare(
            "SELECT id, claimed_by FROM tasks WHERE claimed_by IS NOT NULL AND claimed_at < ?1"
        )?;
        let rows = stmt.query_map(params![cutoff], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    // Release all stale claims
    let rows_affected = conn.execute(
        "UPDATE tasks SET claimed_by = NULL, claimed_at = NULL, status = 'open', updated_at = ?1
         WHERE claimed_by IS NOT NULL AND claimed_at < ?2",
        params![now, cutoff],
    )?;

    // Record history
    for (task_id, agent_id) in &stale_tasks {
        let _ = record_task_history(conn, task_id, Some("in_progress"), "open", agent_id,
            Some(&format!("Released: stale claim (>{}min)", timeout_minutes)));
    }

    Ok(rows_affected)
}

/// Get the next available task (unclaimed, open, highest priority)
pub fn get_next_available_task(conn: &Connection) -> Result<Option<Task>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, status, priority, type, labels, assignee, notes, created_at, updated_at, closed_at, closed_reason
         FROM tasks
         WHERE status = 'open' AND (claimed_by IS NULL OR claimed_by = '')
         ORDER BY priority ASC, created_at ASC
         LIMIT 1"
    )?;

    let mut rows = stmt.query([])?;
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

/// Get all available tasks (unclaimed, open)
pub fn get_available_tasks(conn: &Connection) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, description, status, priority, type, labels, assignee, notes, created_at, updated_at, closed_at, closed_reason
         FROM tasks
         WHERE status = 'open' AND (claimed_by IS NULL OR claimed_by = '')
         ORDER BY priority ASC, created_at ASC"
    )?;

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
