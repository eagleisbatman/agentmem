use anyhow::Result;
use rusqlite::Connection;
use uuid::Uuid;

use crate::config::{load_config, get_config_path};
use crate::db::models::Memory;
use crate::embedding::{create_provider, EmbeddingProvider};
use crate::embedding::qdrant::QdrantStore;

/// Semantic search result with score
#[derive(Debug, Clone)]
pub struct SemanticSearchResult {
    pub memory: Memory,
    pub score: f32,
}

/// Perform semantic search for memories
pub async fn semantic_search(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<SemanticSearchResult>> {
    // Load config
    let config_path = get_config_path();
    if !config_path.exists() {
        anyhow::bail!("Config not found. Run 'am init' first.");
    }
    let config = load_config(&config_path)?;

    // Check if embedding is enabled
    if config.embedding.provider == "none" {
        anyhow::bail!("Embedding not configured. Run 'am init --embedding openai' to enable.");
    }

    // Create embedding provider
    let provider = create_provider(&config.embedding.provider, config.embedding.model.as_deref())?;

    // Generate query embedding
    let query_embedding = provider.embed(query).await?;

    // Search Qdrant
    let store = QdrantStore::new(
        &config.qdrant.url,
        &config.qdrant.collection,
        provider.dimensions(),
    ).await?;

    let search_results = store.search(query_embedding, limit).await?;

    // Fetch full memory details from SQLite
    let mut results = Vec::with_capacity(search_results.len());

    for sr in search_results {
        if let Ok(memory) = get_memory_by_id(conn, &sr.memory_id) {
            results.push(SemanticSearchResult {
                memory,
                score: sr.score,
            });
        }
    }

    Ok(results)
}

/// Check if semantic search is available (embedding configured and Qdrant running)
pub async fn is_semantic_search_available() -> bool {
    let config_path = get_config_path();
    if !config_path.exists() {
        return false;
    }

    let config = match load_config(&config_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    if config.embedding.provider == "none" {
        return false;
    }

    // Try to create provider
    let provider = match create_provider(&config.embedding.provider, config.embedding.model.as_deref()) {
        Ok(p) => p,
        Err(_) => return false,
    };

    // Try to connect to Qdrant
    let store = match QdrantStore::new(
        &config.qdrant.url,
        &config.qdrant.collection,
        provider.dimensions(),
    ).await {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Check health
    store.health_check().await.unwrap_or(false)
}

/// Get a single memory by ID
fn get_memory_by_id(conn: &Connection, memory_id: &str) -> Result<Memory> {
    let mut stmt = conn.prepare(
        "SELECT id, type, title, content, source_chunk, confidence, times_recalled,
         first_observed_at, last_observed_at, last_recalled_at, created_at, updated_at
         FROM memories WHERE id = ?1"
    )?;

    let memory = stmt.query_row([memory_id], |row| {
        let id_str: String = row.get(0)?;
        let id = Uuid::parse_str(&id_str).unwrap_or_default();

        Ok(Memory {
            id,
            memory_type: row.get(1)?,
            title: row.get(2)?,
            content: row.get(3)?,
            source_chunk: row.get(4)?,
            confidence: row.get(5)?,
            times_recalled: row.get(6)?,
            first_observed_at: row.get(7)?,
            last_observed_at: row.get(8)?,
            last_recalled_at: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })?;

    Ok(memory)
}
