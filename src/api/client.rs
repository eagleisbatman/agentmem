use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::api::credentials::get_api_credentials;

/// API client for AgentMem cloud service
pub struct ApiClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl ApiClient {
    /// Create a new API client
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        // Get base URL from env or default
        let base_url = std::env::var("AGENTMEM_API_URL")
            .unwrap_or_else(|_| "https://agentmem.railway.app".to_string());

        // Try to load API key
        let api_key = get_api_credentials().ok().flatten();

        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.api_key.is_some()
    }

    /// Set API key
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Make authenticated request
    async fn request<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<impl Serialize>,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);

        let mut req = self.client.request(method, &url);

        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        if let Some(body) = body {
            req = req.json(&body);
        }

        let response = req.send().await.context("Failed to send request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API error ({}): {}", status, error_text);
        }

        response.json().await.context("Failed to parse response")
    }

    /// GET request
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        self.request::<T>(reqwest::Method::GET, path, None::<()>).await
    }

    /// POST request
    pub async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: B,
    ) -> Result<T> {
        self.request(reqwest::Method::POST, path, Some(body)).await
    }

    /// PUT request
    pub async fn put<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: B,
    ) -> Result<T> {
        self.request(reqwest::Method::PUT, path, Some(body)).await
    }

    /// DELETE request
    pub async fn delete<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        self.request::<T>(reqwest::Method::DELETE, path, None::<()>).await
    }
}

// ============================================================================
// API Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    #[serde(rename = "apiKey", skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStats {
    pub projects: i32,
    pub memories: i32,
    pub sessions: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserWithStats {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub stats: UserStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: Option<String>,
    #[serde(rename = "machineId")]
    pub machine_id: Option<String>,
    #[serde(rename = "lastActiveAt")]
    pub last_active_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
    pub agent: String,
    pub model: Option<String>,
    #[serde(rename = "startedAt")]
    pub started_at: String,
    #[serde(rename = "endedAt")]
    pub ended_at: Option<String>,
    #[serde(rename = "tokensIn")]
    pub tokens_in: i32,
    #[serde(rename = "tokensOut")]
    pub tokens_out: i32,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    #[serde(rename = "projectId")]
    pub project_id: Option<String>,
    pub scope: String,
    #[serde(rename = "memoryType")]
    pub memory_type: String,
    pub title: String,
    pub content: Option<String>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub outcome: String,
    pub confidence: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextResponse {
    pub project: String,
    #[serde(rename = "projectId")]
    pub project_id: Option<String>,
    pub memories: Vec<ContextMemory>,
    pub protected: Vec<ProtectedFile>,
    pub tasks: Vec<ContextTask>,
    pub tools: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextMemory {
    pub id: String,
    pub memory_type: String,
    pub title: String,
    pub content: Option<String>,
    pub scope: String,
    pub agent: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtectedFile {
    pub pattern: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextTask {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: i32,
}

// ============================================================================
// Request/Response types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub path: Option<String>,
    #[serde(rename = "machineId")]
    pub machine_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateSessionRequest {
    #[serde(rename = "projectName")]
    pub project_name: String,
    pub agent: String,
    pub model: Option<String>,
    #[serde(rename = "machineId")]
    pub machine_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateSessionRequest {
    #[serde(rename = "tokensIn", skip_serializing_if = "Option::is_none")]
    pub tokens_in: Option<i32>,
    #[serde(rename = "tokensOut", skip_serializing_if = "Option::is_none")]
    pub tokens_out: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CreateMemoryRequest {
    #[serde(rename = "projectName")]
    pub project_name: String,
    #[serde(rename = "sessionId", skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub scope: String,
    #[serde(rename = "memoryType")]
    pub memory_type: String,
    pub title: String,
    pub content: Option<String>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub confidence: Option<i32>,
    #[serde(rename = "machineId")]
    pub machine_id: Option<String>,
}
