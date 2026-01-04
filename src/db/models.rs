use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String, // open, in_progress, closed
    pub priority: i32,   // 0-4
    pub task_type: String, // bug, feature, task, epic, chore
    pub labels: Vec<String>,
    pub assignee: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub closed_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Memory {
    pub id: Uuid,
    pub memory_type: String, // correction, decision, infrastructure, etc.
    pub title: String,
    pub content: Option<String>,
    pub source_chunk: Option<String>,
    pub confidence: i32,
    pub times_recalled: i32,
    pub first_observed_at: DateTime<Utc>,
    pub last_observed_at: DateTime<Utc>,
    pub last_recalled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtectedFile {
    pub pattern: String,
    pub reason: Option<String>,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub location: String,
    pub description: Option<String>,
    pub usage: Option<String>,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entity {
    pub slug: String,
    pub entity_type: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

// ============================================
// AgentMem 2.0 Models
// ============================================

/// Plan from plan mode - stores implementation plans
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Plan {
    pub id: String,
    pub title: String,
    pub content: Option<String>,
    pub file_path: Option<String>,
    pub status: String, // active, completed, abandoned
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Link between a plan and its tasks
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanTask {
    pub plan_id: String,
    pub task_id: String,
    pub task_order: i32,
    pub created_at: DateTime<Utc>,
}

/// Snapshot of TodoWrite state for session persistence
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TodoWriteSnapshot {
    pub id: String,
    pub session_id: String,
    pub snapshot_json: String, // JSON representation of TodoWrite state
    pub captured_at: DateTime<Utc>,
}

/// Session tracking for continuity across sessions
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub status: String, // active, completed, compacted
    pub agent: Option<String>,
    pub model: Option<String>,
    pub tokens_in: i32,
    pub tokens_out: i32,
    pub last_task_id: Option<String>,
    pub summary: Option<String>,
}

/// Task status change history
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskHistory {
    pub id: String,
    pub task_id: String,
    pub old_status: Option<String>,
    pub new_status: String,
    pub changed_at: DateTime<Utc>,
    pub changed_by: String, // user, agent, hook
    pub notes: Option<String>,
}

