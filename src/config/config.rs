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

pub fn get_agentmem_dir() -> PathBuf {
    PathBuf::from(".agentmem")
}

pub fn get_config_path() -> PathBuf {
    get_agentmem_dir().join("config.yaml")
}

pub fn get_db_path() -> PathBuf {
    get_agentmem_dir().join("agentmem.db")
}

