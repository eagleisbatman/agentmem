use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub embedding: EmbeddingConfig,
    #[serde(default = "default_qdrant_config")]
    pub qdrant: QdrantConfig,
    pub hooks: HooksConfig,
}

fn default_qdrant_config() -> QdrantConfig {
    QdrantConfig {
        url: "http://localhost:6334".to_string(),
        collection: "agentmem_memories".to_string(),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QdrantConfig {
    pub url: String,
    pub collection: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbeddingConfig {
    pub provider: String, // none, ollama, gemini, openai
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HooksConfig {
    pub pre_prompt: HookSettings,
    pub post_session: HookSettings,
    pub post_compact: HookSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HookSettings {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_extract: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_sync: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_reminder: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            embedding: EmbeddingConfig {
                provider: "none".to_string(),
                model: None,
            },
            qdrant: QdrantConfig {
                url: "http://localhost:6334".to_string(),
                collection: "agentmem_memories".to_string(),
            },
            hooks: HooksConfig {
                pre_prompt: HookSettings {
                    enabled: true,
                    timeout_ms: Some(5000),
                    max_tokens: Some(2000),
                    auto_extract: None,
                    auto_sync: None,
                    show_reminder: None,
                },
                post_session: HookSettings {
                    enabled: true,
                    timeout_ms: None,
                    max_tokens: None,
                    auto_extract: Some(true),
                    auto_sync: Some(true),
                    show_reminder: None,
                },
                post_compact: HookSettings {
                    enabled: true,
                    timeout_ms: None,
                    max_tokens: None,
                    auto_extract: None,
                    auto_sync: None,
                    show_reminder: Some(true),
                },
            },
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let content = fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&content)?;
    Ok(config)
}

pub fn save_config<P: AsRef<Path>>(path: P, config: &Config) -> Result<()> {
    let content = serde_yaml::to_string(config)?;
    fs::write(path, content)?;
    Ok(())
}

/// Find the nearest .agentmem directory by walking up the directory tree
/// Returns the path to .agentmem in the current directory or any parent
pub fn find_agentmem_dir() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;

    loop {
        let candidate = current.join(".agentmem");
        if candidate.exists() && candidate.is_dir() {
            return Some(candidate);
        }

        if !current.pop() {
            // Reached root, no .agentmem found
            return None;
        }
    }
}

/// Find workspace root (parent directory containing .agentmem)
/// This enables hierarchical workspace support
pub fn find_workspace_root() -> Option<PathBuf> {
    find_agentmem_dir().and_then(|am_dir| am_dir.parent().map(|p| p.to_path_buf()))
}

/// Get local .agentmem directory (always in current directory)
pub fn get_local_agentmem_dir() -> PathBuf {
    PathBuf::from(".agentmem")
}

/// Get the .agentmem directory - uses hierarchical discovery
/// Falls back to local directory if none found (for init)
pub fn get_agentmem_dir() -> PathBuf {
    find_agentmem_dir().unwrap_or_else(|| PathBuf::from(".agentmem"))
}

/// Check if we're in a workspace (found .agentmem in parent directory)
pub fn is_workspace_subdirectory() -> bool {
    if let Some(am_dir) = find_agentmem_dir() {
        // If .agentmem is not in current directory, we're in a subdirectory
        let local = std::env::current_dir()
            .map(|cwd| cwd.join(".agentmem"))
            .ok();
        local.map(|l| l != am_dir).unwrap_or(false)
    } else {
        false
    }
}

/// Generate a unique project identifier based on directory path
/// Used for Qdrant collection names to isolate projects
pub fn get_project_id() -> String {
    let workspace_root = find_workspace_root()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    // Use the directory name plus a hash of the full path for uniqueness
    let dir_name = workspace_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let path_str = workspace_root.to_string_lossy();
    let hash = simple_hash(&path_str);

    format!("{}_{}", sanitize_collection_name(dir_name), hash)
}

/// Simple hash function for generating collection name suffix
fn simple_hash(s: &str) -> String {
    let mut hash: u32 = 0;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }
    format!("{:08x}", hash)
}

/// Sanitize string for use in Qdrant collection name
fn sanitize_collection_name(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
}

pub fn get_config_path() -> PathBuf {
    get_agentmem_dir().join("config.yaml")
}

pub fn get_db_path() -> PathBuf {
    get_agentmem_dir().join("agentmem.db")
}

