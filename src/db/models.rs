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

