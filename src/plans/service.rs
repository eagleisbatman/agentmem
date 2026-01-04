use rusqlite::{params, Connection, Result};
use crate::db::models::{Plan, PlanTask};
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};
use anyhow::Context;
use serde::{Deserialize, Serialize};

/// Extracted task from plan analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTask {
    pub title: String,
    pub description: String,
    pub priority: i32,
    pub order: i32,
}

/// Result of task extraction
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskExtractionResult {
    pub tasks: Vec<ExtractedTask>,
}

/// Create a new plan
pub fn create_plan(conn: &Connection, title: &str, content: Option<&str>, file_path: Option<&str>) -> Result<String> {
    let suffix: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();
    let id = format!("plan-{}", suffix.to_lowercase());
    let now = Utc::now();

    conn.execute(
        "INSERT INTO plans (id, title, content, file_path, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?6)",
        params![id, title, content, file_path, now, now],
    )?;

    Ok(id)
}

/// Get a plan by ID
pub fn get_plan(conn: &Connection, id: &str) -> Result<Option<Plan>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, content, file_path, status, created_at, updated_at, completed_at
         FROM plans WHERE id = ?1"
    )?;

    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Plan {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            file_path: row.get(3)?,
            status: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
            completed_at: row.get(7)?,
        }))
    } else {
        Ok(None)
    }
}

/// List all plans, optionally filtered by status
pub fn list_plans(conn: &Connection, status_filter: Option<&str>) -> Result<Vec<Plan>> {
    let query = match status_filter {
        Some(_) => "SELECT id, title, content, file_path, status, created_at, updated_at, completed_at
                    FROM plans WHERE status = ?1 ORDER BY created_at DESC",
        None => "SELECT id, title, content, file_path, status, created_at, updated_at, completed_at
                 FROM plans ORDER BY created_at DESC",
    };

    let mut stmt = conn.prepare(query)?;
    let rows = match status_filter {
        Some(status) => stmt.query_map(params![status], row_to_plan)?,
        None => stmt.query_map([], row_to_plan)?,
    };

    let mut plans = Vec::new();
    for plan in rows {
        plans.push(plan?);
    }
    Ok(plans)
}

fn row_to_plan(row: &rusqlite::Row) -> Result<Plan> {
    Ok(Plan {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        file_path: row.get(3)?,
        status: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
        completed_at: row.get(7)?,
    })
}

/// Update plan content
pub fn update_plan(conn: &Connection, id: &str, content: Option<&str>) -> Result<()> {
    let now = Utc::now();
    conn.execute(
        "UPDATE plans SET content = ?1, updated_at = ?2 WHERE id = ?3",
        params![content, now, id],
    )?;
    Ok(())
}

/// Complete a plan
pub fn complete_plan(conn: &Connection, id: &str) -> Result<()> {
    let now = Utc::now();
    conn.execute(
        "UPDATE plans SET status = 'completed', completed_at = ?1, updated_at = ?1 WHERE id = ?2",
        params![now, id],
    )?;
    Ok(())
}

/// Abandon a plan
pub fn abandon_plan(conn: &Connection, id: &str) -> Result<()> {
    let now = Utc::now();
    conn.execute(
        "UPDATE plans SET status = 'abandoned', updated_at = ?1 WHERE id = ?2",
        params![now, id],
    )?;
    Ok(())
}

/// Link a task to a plan
pub fn link_task_to_plan(conn: &Connection, plan_id: &str, task_id: &str, order: i32) -> Result<()> {
    let now = Utc::now();
    conn.execute(
        "INSERT OR REPLACE INTO plan_tasks (plan_id, task_id, task_order, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![plan_id, task_id, order, now],
    )?;

    // Also update the task's plan_id
    conn.execute(
        "UPDATE tasks SET plan_id = ?1 WHERE id = ?2",
        params![plan_id, task_id],
    )?;

    Ok(())
}

/// Get tasks linked to a plan
pub fn get_plan_tasks(conn: &Connection, plan_id: &str) -> Result<Vec<PlanTask>> {
    let mut stmt = conn.prepare(
        "SELECT plan_id, task_id, task_order, created_at
         FROM plan_tasks WHERE plan_id = ?1 ORDER BY task_order"
    )?;

    let rows = stmt.query_map(params![plan_id], |row| {
        Ok(PlanTask {
            plan_id: row.get(0)?,
            task_id: row.get(1)?,
            task_order: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;

    let mut tasks = Vec::new();
    for task in rows {
        tasks.push(task?);
    }
    Ok(tasks)
}

/// Get the active plan (most recent active plan)
pub fn get_active_plan(conn: &Connection) -> Result<Option<Plan>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, content, file_path, status, created_at, updated_at, completed_at
         FROM plans WHERE status = 'active' ORDER BY created_at DESC LIMIT 1"
    )?;

    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Plan {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            file_path: row.get(3)?,
            status: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
            completed_at: row.get(7)?,
        }))
    } else {
        Ok(None)
    }
}

/// The extraction prompt for plan -> tasks
const TASK_EXTRACTION_PROMPT: &str = r#"You are a task extraction assistant. Analyze this implementation plan and extract discrete, actionable tasks.

## Rules:
- Extract clear, specific tasks that can be worked on independently
- Each task should be completable in one session
- Include a brief description of what the task involves
- Assign priority: 1 (high), 2 (medium), 3 (low)
- Order tasks by logical dependency (what needs to be done first)
- Focus on implementation tasks, not research or planning

## Output Format (JSON):
{
  "tasks": [
    {
      "title": "Brief task title (5-10 words)",
      "description": "What this task involves and acceptance criteria",
      "priority": 1,
      "order": 1
    }
  ]
}

If no actionable tasks found, return: {"tasks": []}

## Plan to Analyze:
"#;

/// Extract tasks from a plan using LLM
pub async fn extract_tasks_from_plan(
    plan_content: &str,
    model: &str,
) -> anyhow::Result<TaskExtractionResult> {
    use crate::embedding::openai::chat_completion;

    let user_prompt = format!("{}\n\n{}", TASK_EXTRACTION_PROMPT, plan_content);

    let response = chat_completion(
        model,
        "You are a task extraction assistant. Output valid JSON only.",
        &user_prompt,
    ).await?;

    // Parse the response
    parse_task_extraction_response(&response)
}

/// Parse the GPT response for task extraction
fn parse_task_extraction_response(response: &str) -> anyhow::Result<TaskExtractionResult> {
    // Try direct parse first
    if let Ok(result) = serde_json::from_str::<TaskExtractionResult>(response) {
        return Ok(result);
    }

    // Try to extract JSON from markdown code blocks
    let json_str = if response.contains("```json") {
        response
            .split("```json")
            .nth(1)
            .and_then(|s| s.split("```").next())
            .unwrap_or(response)
            .trim()
    } else if response.contains("```") {
        response
            .split("```")
            .nth(1)
            .unwrap_or(response)
            .trim()
    } else {
        response.trim()
    };

    serde_json::from_str::<TaskExtractionResult>(json_str)
        .context("Failed to parse task extraction response as JSON")
}
