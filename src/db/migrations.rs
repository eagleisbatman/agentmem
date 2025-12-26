use rusqlite::{Connection, Result};

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT DEFAULT 'open',
            priority INTEGER DEFAULT 2,
            type TEXT DEFAULT 'task',
            labels TEXT,
            assignee TEXT,
            notes TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            closed_at DATETIME,
            closed_reason TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_dependencies (
            from_id TEXT NOT NULL,
            to_id TEXT NOT NULL,
            type TEXT DEFAULT 'blocks',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (from_id, to_id, type)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS memories (
            id TEXT PRIMARY KEY,
            type TEXT NOT NULL,
            title TEXT NOT NULL,
            content TEXT,
            source_chunk TEXT,
            confidence INTEGER DEFAULT 70,
            times_recalled INTEGER DEFAULT 0,
            first_observed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            last_observed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            last_recalled_at DATETIME,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS memory_embeddings (
            memory_id TEXT PRIMARY KEY,
            embedding BLOB,
            model TEXT,
            dimensions INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (memory_id) REFERENCES memories(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS memory_entities (
            memory_id TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            entity_name TEXT NOT NULL,
            entity_slug TEXT NOT NULL,
            FOREIGN KEY (memory_id) REFERENCES memories(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS entities (
            slug TEXT PRIMARY KEY,
            type TEXT NOT NULL,
            name TEXT NOT NULL,
            aliases TEXT,
            metadata TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS session_recalls (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            memory_id TEXT NOT NULL,
            query TEXT,
            similarity REAL,
            source TEXT,
            feedback INTEGER,
            recalled_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (memory_id) REFERENCES memories(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS protected_files (
            pattern TEXT PRIMARY KEY,
            reason TEXT,
            added_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tools (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            location TEXT NOT NULL,
            description TEXT,
            usage TEXT,
            added_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS config (
            key TEXT PRIMARY KEY,
            value TEXT
        )",
        [],
    )?;

    // Indexes
    conn.execute("CREATE INDEX IF NOT EXISTS idx_memories_type ON memories(type)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_memories_last_recalled ON memories(last_recalled_at)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_memory_entities_slug ON memory_entities(entity_slug)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority)", [])?;

    Ok(())
}

