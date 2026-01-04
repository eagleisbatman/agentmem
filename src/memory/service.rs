use rusqlite::{params, Connection, Result};
use crate::db::models::{Memory, ProtectedFile};
use crate::config::{load_config, get_config_path};
use crate::embedding::create_provider;
use crate::embedding::qdrant::QdrantStore;
use chrono::Utc;
use uuid::Uuid;
use anyhow::Result as AnyhowResult;

/// Add a memory to SQLite (sync, no embedding)
pub fn add_memory(
    conn: &Connection,
    memory_type: &str,
    title: &str,
    content: Option<&str>,
) -> Result<Uuid> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    conn.execute(
        "INSERT INTO memories (id, type, title, content, first_observed_at, last_observed_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id.to_string(), memory_type, title, content, now, now, now, now],
    )?;

    Ok(id)
}

/// Add a memory with embedding (async, stores in both SQLite and Qdrant)
pub async fn add_memory_with_embedding(
    conn: &Connection,
    memory_type: &str,
    title: &str,
    content: Option<&str>,
) -> AnyhowResult<(Uuid, bool)> {
    // First, add to SQLite
    let id = add_memory(conn, memory_type, title, content)
        .map_err(|e| anyhow::anyhow!("Failed to add memory to SQLite: {}", e))?;

    // Try to add embedding if configured
    let embedded = match try_embed_memory(&id.to_string(), memory_type, title, content).await {
        Ok(_) => true,
        Err(e) => {
            // Log but don't fail - embedding is optional
            eprintln!("Warning: Failed to embed memory: {}", e);
            false
        }
    };

    Ok((id, embedded))
}

/// Try to generate and store embedding for a memory
async fn try_embed_memory(
    memory_id: &str,
    memory_type: &str,
    title: &str,
    content: Option<&str>,
) -> AnyhowResult<()> {
    // Load config
    let config_path = get_config_path();
    if !config_path.exists() {
        anyhow::bail!("Config not found");
    }
    let config = load_config(&config_path)?;

    // Check if embedding is enabled
    if config.embedding.provider == "none" {
        return Ok(()); // Silently skip if not configured
    }

    // Create embedding provider
    let provider = create_provider(&config.embedding.provider, config.embedding.model.as_deref())?;

    // Create text to embed (title + content)
    let text = match content {
        Some(c) => format!("{}: {}", title, c),
        None => title.to_string(),
    };

    // Generate embedding
    let embedding = provider.embed(&text).await?;

    // Store in Qdrant
    let store = QdrantStore::new(
        &config.qdrant.url,
        &config.qdrant.collection,
        provider.dimensions(),
    ).await?;

    store.upsert(memory_id, embedding, memory_type, title).await?;

    Ok(())
}

pub fn list_memories(conn: &Connection) -> Result<Vec<Memory>> {
    let mut stmt = conn.prepare("SELECT id, type, title, content, source_chunk, confidence, times_recalled, first_observed_at, last_observed_at, last_recalled_at, created_at, updated_at FROM memories")?;
    let memory_iter = stmt.query_map([], |row| {
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

    let mut memories = Vec::new();
    for memory in memory_iter {
        memories.push(memory?);
    }
    Ok(memories)
}

pub fn add_protected_file(conn: &Connection, pattern: &str, reason: Option<&str>) -> Result<()> {
    let now = Utc::now();
    conn.execute(
        "INSERT INTO protected_files (pattern, reason, added_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(pattern) DO UPDATE SET reason = excluded.reason, added_at = excluded.added_at",
        params![pattern, reason, now],
    )?;
    Ok(())
}

pub fn add_tool(conn: &Connection, location: &str, name: &str, description: Option<&str>, usage: Option<&str>) -> Result<()> {
    let now = Utc::now();
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO tools (id, name, location, description, usage, added_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, name, location, description, usage, now],
    )?;
    Ok(())
}

pub fn list_protected_files(conn: &Connection) -> Result<Vec<ProtectedFile>> {
    let mut stmt = conn.prepare("SELECT pattern, reason, added_at FROM protected_files")?;
    let file_iter = stmt.query_map([], |row| {
        Ok(ProtectedFile {
            pattern: row.get(0)?,
            reason: row.get(1)?,
            added_at: row.get(2)?,
        })
    })?;

    let mut files = Vec::new();
    for file in file_iter {
        files.push(file?);
    }
    Ok(files)
}
