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

    // ============================================
    // AgentMem 2.0 Tables
    // ============================================

    // Plans table - Store plans from plan mode
    conn.execute(
        "CREATE TABLE IF NOT EXISTS plans (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT,
            file_path TEXT,
            status TEXT DEFAULT 'active',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            completed_at DATETIME
        )",
        [],
    )?;

    // Plan-Tasks link table - Link plans to their tasks
    conn.execute(
        "CREATE TABLE IF NOT EXISTS plan_tasks (
            plan_id TEXT NOT NULL,
            task_id TEXT NOT NULL,
            task_order INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (plan_id, task_id),
            FOREIGN KEY (plan_id) REFERENCES plans(id),
            FOREIGN KEY (task_id) REFERENCES tasks(id)
        )",
        [],
    )?;

    // TodoWrite snapshots - Capture TodoWrite state for persistence
    conn.execute(
        "CREATE TABLE IF NOT EXISTS todowrite_snapshots (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            snapshot_json TEXT NOT NULL,
            captured_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (session_id) REFERENCES sessions(id)
        )",
        [],
    )?;

    // Sessions table - Track session continuity
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            ended_at DATETIME,
            status TEXT DEFAULT 'active',
            agent TEXT,
            model TEXT,
            tokens_in INTEGER DEFAULT 0,
            tokens_out INTEGER DEFAULT 0,
            last_task_id TEXT,
            summary TEXT,
            FOREIGN KEY (last_task_id) REFERENCES tasks(id)
        )",
        [],
    )?;

    // Task history - Track status changes over time
    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_history (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            old_status TEXT,
            new_status TEXT NOT NULL,
            changed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            changed_by TEXT DEFAULT 'agent',
            notes TEXT,
            FOREIGN KEY (task_id) REFERENCES tasks(id)
        )",
        [],
    )?;

    // Add plan_id column to tasks (for linking tasks to plans)
    // Using ALTER TABLE with IF NOT EXISTS pattern for SQLite
    let _ = conn.execute("ALTER TABLE tasks ADD COLUMN plan_id TEXT REFERENCES plans(id)", []);

    // Add parent_task_id column to tasks (for subtasks)
    let _ = conn.execute("ALTER TABLE tasks ADD COLUMN parent_task_id TEXT REFERENCES tasks(id)", []);

    // ============================================
    // Indexes
    // ============================================
    conn.execute("CREATE INDEX IF NOT EXISTS idx_memories_type ON memories(type)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_memories_last_recalled ON memories(last_recalled_at)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_memory_entities_slug ON memory_entities(entity_slug)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority)", [])?;

    // New indexes for 2.0 tables
    conn.execute("CREATE INDEX IF NOT EXISTS idx_plans_status ON plans(status)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_sessions_started ON sessions(started_at DESC)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_task_history_task ON task_history(task_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_todowrite_session ON todowrite_snapshots(session_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_tasks_plan ON tasks(plan_id)", [])?;

    Ok(())
}

