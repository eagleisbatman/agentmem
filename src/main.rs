pub mod api;
pub mod config;
pub mod db;
pub mod embedding;
pub mod hooks;
pub mod init;
pub mod mcp;
pub mod memory;
pub mod plans;
pub mod retrieval;
pub mod sessions;
pub mod sync;
pub mod tasks;

use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use crate::init::run_init;
use crate::config::get_db_path;
use crate::db::get_connection;
use crate::tasks::service::{create_task, list_tasks, get_ready_tasks, claim_task, release_task, get_next_available_task, get_available_tasks, release_all_agent_tasks, release_stale_claims};
use crate::memory::service::{add_memory_with_embedding, list_memories, add_protected_file, add_tool};
use crate::sync::{export_to_jsonl, import_from_jsonl, git_sync};
use crate::retrieval::context::{get_context_async, format_context_markdown};
use crate::retrieval::search::semantic_search;
use crate::hooks::{install_hooks, list_hooks, test_hook, detect_installed_agents};
use crate::memory::extraction::{extract_from_transcript, extract_and_store, read_transcript_file};
use std::path::Path;

#[derive(Parser)]
#[command(name = "am")]
#[command(about = "Agent Memory CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize AgentMem in current project
    Init {
        #[arg(long)]
        quiet: bool,
        #[arg(long)]
        embedding: Option<String>,
        #[arg(long)]
        model: Option<String>,
    },
    /// Task management
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },
    /// Memory management
    Mem {
        #[command(subcommand)]
        command: MemoryCommands,
    },
    /// Mark file as protected
    Protect {
        path: String,
        reason: Option<String>,
    },
    /// Register a script/utility
    Tool {
        location: String,
        description: String,
        usage: Option<String>,
    },
    /// Get context for a query
    Context {
        #[arg(long)]
        query: Option<String>,
        #[arg(long)]
        task: Option<String>,
        #[arg(long)]
        file: Option<String>,
        #[arg(long, default_value_t = 5)]
        limit_memories: usize,
        #[arg(long, default_value_t = 3)]
        limit_tasks: usize,
        #[arg(long, default_value = "markdown")]
        format: String,
        #[arg(long)]
        json: bool,
    },
    /// Sync with git
    Sync {
        #[arg(long)]
        push: bool,
        #[arg(long)]
        message: Option<String>,
    },
    /// Export to JSONL
    Export {
        #[arg(short, long)]
        path: Option<String>,
    },
    /// Import from JSONL
    Import {
        #[arg(short, long)]
        path: Option<String>,
        /// Regenerate embeddings for imported memories
        #[arg(long)]
        embed: bool,
    },
    /// Manage hooks for AI agents
    Hook {
        #[command(subcommand)]
        command: HookCommands,
    },
    /// Extract memories from session transcript
    Extract {
        /// Path to transcript file (JSONL or plain text)
        #[arg(long)]
        transcript: String,
        /// LLM model to use for extraction
        #[arg(long, default_value = "gpt-4o")]
        model: String,
        /// Skip deduplication
        #[arg(long)]
        no_dedupe: bool,
        /// Dry run - show what would be extracted without storing
        #[arg(long)]
        dry_run: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Check system health and dependencies
    Doctor {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Manage cloud authentication
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    /// Manage cloud sessions (for hooks)
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },
    /// Manage plans
    Plan {
        #[command(subcommand)]
        command: PlanCommands,
    },
    /// Run MCP server (for Claude Code plugin)
    McpServer,
}

#[derive(Subcommand)]
enum TaskCommands {
    Create {
        title: String,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long, default_value_t = 2)]
        priority: i32,
        #[arg(short, long, default_value = "task")]
        task_type: String,
    },
    List {
        #[arg(long)]
        json: bool,
    },
    Ready {
        #[arg(long)]
        json: bool,
    },
    /// Update task status (open, in_progress, closed)
    Update {
        id: String,
        #[arg(long)]
        status: String,
        #[arg(long)]
        notes: Option<String>,
    },
    /// Show task history
    History {
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Show task details
    Show {
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Claim a task for an agent (sub-agent coordination)
    Claim {
        id: String,
        #[arg(long)]
        agent: String,
    },
    /// Release a claimed task
    Release {
        id: String,
        #[arg(long)]
        agent: String,
    },
    /// Get and claim the next available task
    Next {
        #[arg(long)]
        agent: String,
        #[arg(long)]
        json: bool,
    },
    /// List available (unclaimed) tasks
    Available {
        #[arg(long)]
        json: bool,
    },
    /// Release all tasks claimed by an agent (session cleanup)
    ReleaseAll {
        #[arg(long)]
        agent: String,
    },
    /// Release stale claims (tasks claimed too long ago)
    CleanupStale {
        /// Timeout in minutes (default: 30)
        #[arg(long, default_value_t = 30)]
        timeout: i64,
    },
}

#[derive(Subcommand)]
enum MemoryCommands {
    Add {
        memory_type: String,
        title: String,
        #[arg(short, long)]
        content: Option<String>,
        /// Skip syncing to cloud (local only)
        #[arg(long)]
        local: bool,
    },
    List {
        #[arg(long)]
        json: bool,
        /// Include cloud memories
        #[arg(long)]
        cloud: bool,
    },
    Search {
        query: String,
    },
    /// Sync all local memories to cloud
    Push,
}

#[derive(Subcommand)]
enum HookCommands {
    /// Install hooks for an AI agent
    Install {
        /// Agent: claude-code, gemini-cli, codex-cli, cursor
        agent: String,
    },
    /// List installed hooks
    List {
        #[arg(long)]
        json: bool,
    },
    /// Test hook execution
    Test {
        /// Hook type: pre-prompt, post-session
        hook_type: String,
    },
    /// Detect which agents are installed on this system
    Detect {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Login with API key
    Login {
        /// API key (or enter interactively)
        #[arg(long)]
        api_key: Option<String>,
    },
    /// Register a new account
    Register {
        /// Email address
        #[arg(long)]
        email: Option<String>,
        /// Your name
        #[arg(long)]
        name: Option<String>,
    },
    /// Show current authentication status
    Status,
    /// Logout and clear credentials
    Logout,
}

#[derive(Subcommand)]
enum SessionCommands {
    /// Start a new session (called by hooks)
    Start {
        /// Agent name (claude-code, gemini-cli, etc.)
        #[arg(long)]
        agent: Option<String>,
        /// Model name
        #[arg(long)]
        model: Option<String>,
    },
    /// End current session (called by hooks)
    End {
        /// Input tokens used
        #[arg(long)]
        tokens_in: Option<i32>,
        /// Output tokens used
        #[arg(long)]
        tokens_out: Option<i32>,
        /// Model name
        #[arg(long)]
        model: Option<String>,
    },
    /// Show current session status
    Status,
    /// List recent sessions
    List {
        #[arg(long, default_value_t = 10)]
        limit: i32,
        #[arg(long)]
        json: bool,
    },
    /// Save TodoWrite state
    SaveTodos {
        /// JSON representation of TodoWrite state
        snapshot_json: String,
    },
    /// Get latest TodoWrite state
    GetTodos {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum PlanCommands {
    /// Create a new plan
    Create {
        title: String,
        #[arg(short, long)]
        content: Option<String>,
        #[arg(long)]
        file: Option<String>,
    },
    /// List all plans
    List {
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Show plan details
    Show {
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Complete a plan
    Complete {
        id: String,
    },
    /// Abandon a plan
    Abandon {
        id: String,
    },
    /// Link a task to a plan
    Link {
        plan_id: String,
        task_id: String,
        #[arg(long, default_value_t = 0)]
        order: i32,
    },
    /// Get the active plan
    Active {
        #[arg(long)]
        json: bool,
    },
    /// Extract tasks from a plan file using LLM
    ExtractTasks {
        /// Path to plan file (markdown)
        #[arg(long)]
        file: String,
        /// Plan ID to link tasks to (uses active plan if not specified)
        #[arg(long)]
        plan_id: Option<String>,
        /// LLM model for extraction
        #[arg(long, default_value = "gpt-4o")]
        model: String,
        /// Dry run - show tasks without creating
        #[arg(long)]
        dry_run: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { quiet, embedding, model } => {
            run_init(quiet, embedding, model)?;
        },
        Commands::Task { command } => {
            let db_path = get_db_path();
            if !db_path.exists() {
                anyhow::bail!("AgentMem not initialized. Run 'am init' first.");
            }
            let conn = get_connection(db_path)?;
            match command {
                TaskCommands::Create { title, description, priority, task_type } => {
                    let id = create_task(&conn, &title, description.as_deref(), priority, &task_type)?;
                    println!("✓ Created: {} \"{}\"", id, title);
                },
                TaskCommands::List { json } => {
                    let tasks = list_tasks(&conn)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&tasks)?);
                    } else {
                        for t in tasks {
                            println!("[P{}] {}: {} ({})", t.priority, t.id, t.title, t.status);
                        }
                    }
                },
                TaskCommands::Ready { json } => {
                    let tasks = get_ready_tasks(&conn)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&tasks)?);
                    } else {
                        for t in tasks {
                            println!("[P{}] {}: {} ({})", t.priority, t.id, t.title, t.status);
                        }
                    }
                },
                TaskCommands::Update { id, status, notes } => {
                    crate::tasks::service::update_task_status(&conn, &id, &status, "user", notes.as_deref())?;
                    println!("Updated task {} to status: {}", id, status);
                },
                TaskCommands::History { id, json } => {
                    let history = crate::tasks::service::get_task_history(&conn, &id)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&history)?);
                    } else {
                        if history.is_empty() {
                            println!("No history for task: {}", id);
                        } else {
                            println!("History for {}:", id);
                            for h in history {
                                println!("  {} -> {} (by {} at {})",
                                    h.old_status.as_deref().unwrap_or("created"),
                                    h.new_status,
                                    h.changed_by,
                                    h.changed_at.format("%Y-%m-%d %H:%M")
                                );
                            }
                        }
                    }
                },
                TaskCommands::Show { id, json } => {
                    if let Some(task) = crate::tasks::service::get_task(&conn, &id)? {
                        if json {
                            println!("{}", serde_json::to_string_pretty(&task)?);
                        } else {
                            println!("Task: {} ({})", task.title, task.status);
                            println!("  ID: {}", task.id);
                            println!("  Priority: {}", task.priority);
                            println!("  Type: {}", task.task_type);
                            if let Some(d) = task.description {
                                println!("  Description: {}", d);
                            }
                            if let Some(n) = task.notes {
                                println!("  Notes: {}", n);
                            }
                        }
                    } else {
                        println!("Task not found: {}", id);
                    }
                },
                TaskCommands::Claim { id, agent } => {
                    match claim_task(&conn, &id, &agent)? {
                        true => println!("✓ Task {} claimed by agent {}", id, agent),
                        false => println!("✗ Task {} is already claimed by another agent", id),
                    }
                },
                TaskCommands::Release { id, agent } => {
                    match release_task(&conn, &id, &agent)? {
                        true => println!("✓ Task {} released by agent {}", id, agent),
                        false => println!("✗ Task {} is not claimed by agent {}", id, agent),
                    }
                },
                TaskCommands::Next { agent, json } => {
                    // Retry loop to handle race conditions
                    let mut attempts = 0;
                    const MAX_ATTEMPTS: u32 = 5;

                    loop {
                        attempts += 1;
                        if let Some(task) = get_next_available_task(&conn)? {
                            // Try to claim the task - may fail if another agent claimed it first
                            if claim_task(&conn, &task.id, &agent)? {
                                if json {
                                    println!("{}", serde_json::to_string_pretty(&task)?);
                                } else {
                                    println!("✓ Claimed task: {} \"{}\"", task.id, task.title);
                                    if let Some(d) = &task.description {
                                        println!("  Description: {}", d);
                                    }
                                }
                                break;
                            } else if attempts >= MAX_ATTEMPTS {
                                // All attempts failed due to race conditions
                                if json {
                                    println!("null");
                                } else {
                                    println!("✗ Failed to claim any task after {} attempts (high contention)", attempts);
                                }
                                break;
                            }
                            // Race condition - another agent claimed it, retry with next available
                        } else {
                            // No tasks available
                            if json {
                                println!("null");
                            } else {
                                println!("No available tasks");
                            }
                            break;
                        }
                    }
                },
                TaskCommands::Available { json } => {
                    let tasks = get_available_tasks(&conn)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&tasks)?);
                    } else {
                        if tasks.is_empty() {
                            println!("No available tasks");
                        } else {
                            println!("Available tasks ({}):", tasks.len());
                            for t in tasks {
                                println!("  [P{}] {}: {}", t.priority, t.id, t.title);
                            }
                        }
                    }
                },
                TaskCommands::ReleaseAll { agent } => {
                    let count = release_all_agent_tasks(&conn, &agent)?;
                    if count > 0 {
                        println!("✓ Released {} task(s) claimed by agent {}", count, agent);
                    } else {
                        println!("No tasks claimed by agent {}", agent);
                    }
                },
                TaskCommands::CleanupStale { timeout } => {
                    let count = release_stale_claims(&conn, timeout)?;
                    if count > 0 {
                        println!("✓ Released {} stale claim(s) (older than {} minutes)", count, timeout);
                    } else {
                        println!("No stale claims found");
                    }
                },
            }
        },
        Commands::Mem { command } => {
            let db_path = get_db_path();
            if !db_path.exists() {
                anyhow::bail!("AgentMem not initialized. Run 'am init' first.");
            }
            let conn = get_connection(db_path)?;
            match command {
                MemoryCommands::Add { memory_type, title, content, local } => {
                    let (id, embedded) = add_memory_with_embedding(&conn, &memory_type, &title, content.as_deref()).await?;
                    if embedded {
                        println!("✓ Added memory with embedding: {} \"{}\"", id, title);
                    } else {
                        println!("✓ Added memory: {} \"{}\"", id, title);
                    }

                    // Sync to cloud if authenticated and not --local
                    if !local {
                        if let Ok(Some(_)) = crate::api::get_api_credentials() {
                            match sync_memory_to_cloud(&memory_type, &title, content.as_deref()).await {
                                Ok(_) => println!("  ↑ Synced to cloud"),
                                Err(e) => println!("  ! Cloud sync failed: {}", e),
                            }
                        }
                    }
                },
                MemoryCommands::List { json, cloud } => {
                    let memories = list_memories(&conn)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&memories)?);
                    } else {
                        if !memories.is_empty() {
                            println!("Local memories:");
                            for m in memories {
                                println!("  [{}] {}: {}", m.memory_type, m.id, m.title);
                            }
                        } else {
                            println!("No local memories.");
                        }

                        // Show cloud memories if requested
                        if cloud {
                            if let Ok(Some(_)) = crate::api::get_api_credentials() {
                                match list_cloud_memories().await {
                                    Ok(cloud_mems) => {
                                        println!("\nCloud memories:");
                                        if cloud_mems.is_empty() {
                                            println!("  No cloud memories.");
                                        } else {
                                            for m in cloud_mems {
                                                println!("  [{}] {}: {} ({})",
                                                    m.memory_type, m.id, m.title, m.scope);
                                            }
                                        }
                                    },
                                    Err(e) => println!("\n! Failed to fetch cloud memories: {}", e),
                                }
                            } else {
                                println!("\n! Not authenticated. Run 'am auth login' first.");
                            }
                        }
                    }
                },
                MemoryCommands::Search { query } => {
                    match semantic_search(&conn, &query, 10).await {
                        Ok(results) => {
                            if results.is_empty() {
                                println!("No memories found for query: \"{}\"", query);
                            } else {
                                println!("Found {} memories:\n", results.len());
                                for r in results {
                                    println!("[{:.2}] [{}] {}: {}",
                                        r.score,
                                        r.memory.memory_type,
                                        r.memory.title,
                                        r.memory.content.as_deref().unwrap_or("")
                                    );
                                }
                            }
                        },
                        Err(e) => {
                            println!("Semantic search failed: {}", e);
                            println!("Hint: Make sure Qdrant is running and embedding is configured.");
                        }
                    }
                },
                MemoryCommands::Push => {
                    if let Ok(Some(_)) = crate::api::get_api_credentials() {
                        let memories = list_memories(&conn)?;
                        if memories.is_empty() {
                            println!("No local memories to push.");
                        } else {
                            println!("Pushing {} memories to cloud...", memories.len());
                            let mut success = 0;
                            let mut failed = 0;
                            for m in &memories {
                                match sync_memory_to_cloud(&m.memory_type, &m.title, m.content.as_deref()).await {
                                    Ok(_) => success += 1,
                                    Err(_) => failed += 1,
                                }
                            }
                            println!("✓ Pushed {} memories ({} failed)", success, failed);
                        }
                    } else {
                        anyhow::bail!("Not authenticated. Run 'am auth login' first.");
                    }
                },
            }
        },
        Commands::Protect { path, reason } => {
            let db_path = get_db_path();
            if !db_path.exists() {
                anyhow::bail!("AgentMem not initialized. Run 'am init' first.");
            }
            let conn = get_connection(db_path)?;
            add_protected_file(&conn, &path, reason.as_deref())?;
            println!("✓ Protected: {}", path);
        },
        Commands::Tool { location, description, usage } => {
            let db_path = get_db_path();
            if !db_path.exists() {
                anyhow::bail!("AgentMem not initialized. Run 'am init' first.");
            }
            let conn = get_connection(db_path)?;
            let name = Path::new(&location)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&location);
            add_tool(&conn, &location, name, Some(&description), usage.as_deref())?;
            println!("✓ Registered tool: {}", name);
        },
        Commands::Context { query, task, file, limit_memories, limit_tasks, format, json } => {
            let db_path = get_db_path();
            if !db_path.exists() {
                anyhow::bail!("AgentMem not initialized. Run 'am init' first.");
            }
            let conn = get_connection(db_path)?;

            // Use async version which tries semantic search first
            let context = get_context_async(&conn, query.as_deref(), task.as_deref(), file.as_deref(), limit_memories, limit_tasks).await?;

            if json || format == "json" {
                println!("{}", serde_json::to_string_pretty(&context)?);
            } else if format == "markdown" {
                println!("{}", format_context_markdown(&context));
            } else {
                println!("{}", format_context_markdown(&context));
            }
        },
        Commands::Sync { push, message } => {
            let db_path = get_db_path();
            if !db_path.exists() {
                anyhow::bail!("AgentMem not initialized. Run 'am init' first.");
            }
            let conn = get_connection(db_path)?;
            export_to_jsonl(&conn, ".agentmem/agentmem.jsonl")?;
            match git_sync(push, message.as_deref())? {
                crate::sync::GitSyncResult::Synced => println!("✓ Synced with git"),
                crate::sync::GitSyncResult::SyncedWithPull => {
                    // Import any changes that came from the pull
                    println!("✓ Pulled remote changes");
                    import_from_jsonl(&conn, ".agentmem/agentmem.jsonl")?;
                    println!("✓ Imported remote memories");
                    // Re-export to merge local + remote
                    export_to_jsonl(&conn, ".agentmem/agentmem.jsonl")?;
                    println!("✓ Synced with git");
                }
                crate::sync::GitSyncResult::NoChanges => println!("✓ No changes to sync"),
                crate::sync::GitSyncResult::NotAGitRepo => println!("⚠ Not a git repository - memories saved locally only"),
                crate::sync::GitSyncResult::PullConflict => {
                    println!("⚠ Pull conflict - please resolve manually:");
                    println!("  1. Run: git pull --rebase");
                    println!("  2. Resolve conflicts in .agentmem/agentmem.jsonl");
                    println!("  3. Run: am import");
                    println!("  4. Run: am sync --push");
                }
            }
        },
        Commands::Export { path } => {
            let db_path = get_db_path();
            if !db_path.exists() {
                anyhow::bail!("AgentMem not initialized. Run 'am init' first.");
            }
            let conn = get_connection(db_path)?;
            let export_path = path.unwrap_or_else(|| ".agentmem/agentmem.jsonl".to_string());
            export_to_jsonl(&conn, export_path)?;
            println!("✓ Exported to JSONL");
        },
        Commands::Import { path, embed } => {
            let db_path = get_db_path();
            let conn = get_connection(db_path)?;
            let import_path = path.unwrap_or_else(|| ".agentmem/agentmem.jsonl".to_string());
            import_from_jsonl(&conn, &import_path)?;
            println!("✓ Imported from JSONL: {}", import_path);

            if embed {
                println!("Regenerating embeddings...");
                let (embedded, total) = crate::memory::service::regenerate_all_embeddings(&conn).await?;
                println!("✓ Regenerated embeddings: {}/{} memories", embedded, total);
            }
        },
        Commands::Hook { command } => {
            let db_path = get_db_path();
            if !db_path.exists() {
                anyhow::bail!("AgentMem not initialized. Run 'am init' first.");
            }
            match command {
                HookCommands::Install { agent } => {
                    install_hooks(&agent)?;
                },
                HookCommands::List { json } => {
                    let hooks = list_hooks()?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&hooks)?);
                    } else {
                        if hooks.is_empty() {
                            println!("No hooks installed. Run 'am hook install <agent>' first.");
                        } else {
                            for h in hooks {
                                println!("{}: {} ({})", h.name, h.path, h.hook_type);
                            }
                        }
                    }
                },
                HookCommands::Test { hook_type } => {
                    let result = test_hook(&hook_type)?;
                    println!("{}", result);
                },
                HookCommands::Detect { json } => {
                    let agents = detect_installed_agents();
                    if json {
                        let agent_names: Vec<_> = agents.iter().map(|a| a.cli_name()).collect();
                        println!("{}", serde_json::to_string_pretty(&agent_names)?);
                    } else {
                        if agents.is_empty() {
                            println!("No supported AI agents detected.");
                            println!("\nSupported agents:");
                            println!("  - claude-code (Claude Code CLI)");
                            println!("  - gemini-cli (Gemini CLI)");
                            println!("  - codex-cli (Codex CLI)");
                            println!("  - cursor (Cursor IDE)");
                        } else {
                            println!("Detected AI agents:\n");
                            for agent in &agents {
                                println!("  {} ({})", agent.display_name(), agent.cli_name());
                            }
                            println!("\nTo install hooks:");
                            for agent in &agents {
                                println!("  am hook install {}", agent.cli_name());
                            }
                        }
                    }
                },
            }
        },
        Commands::Extract { transcript, model, no_dedupe, dry_run, json } => {
            let db_path = get_db_path();
            if !db_path.exists() {
                anyhow::bail!("AgentMem not initialized. Run 'am init' first.");
            }

            // Read transcript file
            let transcript_content = read_transcript_file(&transcript)?;

            if dry_run {
                // Dry run - just show what would be extracted
                let result = extract_from_transcript(&transcript_content, &model).await?;

                if json {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                } else {
                    if result.memories.is_empty() {
                        println!("No memories extracted from transcript.");
                    } else {
                        println!("Would extract {} memories:\n", result.memories.len());
                        for m in &result.memories {
                            println!("[{}] {} (confidence: {})", m.memory_type, m.title, m.confidence);
                            println!("  Content: {}", m.content);
                            println!("  Reasoning: {}", m.reasoning);
                            println!();
                        }
                    }
                }
            } else {
                // Actually extract and store
                let conn = get_connection(db_path)?;
                let stats = extract_and_store(&conn, &transcript_content, &model, !no_dedupe).await?;

                if json {
                    println!("{}", serde_json::json!({
                        "extracted": stats.extracted,
                        "stored": stats.stored,
                        "duplicates": stats.duplicates
                    }));
                } else {
                    println!("Extracted {} memories from transcript.", stats.extracted);
                    println!("Stored {} new memories ({} duplicates skipped).", stats.stored, stats.duplicates);
                }
            }
        },
        Commands::Doctor { json } => {
            run_doctor(json).await?;
        },
        Commands::Auth { command } => {
            run_auth(command).await?;
        },
        Commands::Session { command } => {
            run_session(command).await?;
        },
        Commands::Plan { command } => {
            run_plan(command).await?;
        },
        Commands::McpServer => {
            crate::mcp::server::run_server()?;
        },
    }

    Ok(())
}

/// Run health check diagnostics
async fn run_doctor(json_output: bool) -> Result<()> {
    use std::process::Command;

    #[derive(serde::Serialize)]
    struct HealthCheck {
        component: String,
        status: String,
        details: Option<String>,
    }

    let mut checks: Vec<HealthCheck> = Vec::new();

    if !json_output {
        println!();
        println!("AgentMem Health Check");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
    }

    // Check 1: .agentmem directory
    let am_dir = crate::config::get_agentmem_dir();
    let am_exists = am_dir.exists();
    checks.push(HealthCheck {
        component: "Project initialized".to_string(),
        status: if am_exists { "ok" } else { "missing" }.to_string(),
        details: Some(am_dir.display().to_string()),
    });
    if !json_output {
        println!("  {} Project initialized ({})",
            if am_exists { "✓" } else { "✗" },
            am_dir.display());
    }

    // Check 2: Database
    let db_path = crate::config::get_db_path();
    let db_exists = db_path.exists();
    checks.push(HealthCheck {
        component: "Database".to_string(),
        status: if db_exists { "ok" } else { "missing" }.to_string(),
        details: Some(db_path.display().to_string()),
    });
    if !json_output {
        println!("  {} Database ({})",
            if db_exists { "✓" } else { "✗" },
            db_path.display());
    }

    // Check 3: Docker installed
    let docker_installed = Command::new("docker")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    checks.push(HealthCheck {
        component: "Docker".to_string(),
        status: if docker_installed { "ok" } else { "missing" }.to_string(),
        details: None,
    });
    if !json_output {
        println!("  {} Docker installed",
            if docker_installed { "✓" } else { "✗" });
    }

    // Check 4: Docker running
    let docker_running = docker_installed && Command::new("docker")
        .arg("info")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if docker_installed {
        checks.push(HealthCheck {
            component: "Docker daemon".to_string(),
            status: if docker_running { "ok" } else { "not running" }.to_string(),
            details: None,
        });
        if !json_output {
            println!("  {} Docker daemon running",
                if docker_running { "✓" } else { "✗" });
        }
    }

    // Check 5: Qdrant container
    let qdrant_running = docker_running && Command::new("docker")
        .args(["ps", "--format", "{{.Names}}"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .any(|name| name == "agentmem-qdrant")
        })
        .unwrap_or(false);
    checks.push(HealthCheck {
        component: "Qdrant".to_string(),
        status: if qdrant_running { "ok" } else { "not running" }.to_string(),
        details: Some("localhost:6334".to_string()),
    });
    if !json_output {
        println!("  {} Qdrant container (localhost:6334)",
            if qdrant_running { "✓" } else { "✗" });
    }

    // Check 6: OpenAI API key
    let openai_key = std::env::var("OPENAI_API_KEY").ok()
        .or_else(|| {
            let creds_path = dirs::home_dir()?.join(".agentmem").join("credentials");
            std::fs::read_to_string(creds_path).ok()
                .and_then(|content| {
                    content.lines()
                        .find(|l| l.starts_with("OPENAI_API_KEY="))
                        .map(|l| l.strip_prefix("OPENAI_API_KEY=").unwrap_or("").to_string())
                })
        });
    let has_openai = openai_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false);
    checks.push(HealthCheck {
        component: "OpenAI API key".to_string(),
        status: if has_openai { "ok" } else { "missing" }.to_string(),
        details: if has_openai { Some("configured".to_string()) } else { None },
    });
    if !json_output {
        println!("  {} OpenAI API key",
            if has_openai { "✓" } else { "✗" });
    }

    // Check 7: Hooks installed
    let hooks_dir = am_dir.join("hooks");
    let hooks_count = if hooks_dir.exists() {
        std::fs::read_dir(&hooks_dir)
            .map(|entries| entries.filter(|e| e.is_ok()).count())
            .unwrap_or(0)
    } else {
        0
    };
    checks.push(HealthCheck {
        component: "Hooks".to_string(),
        status: if hooks_count > 0 { "ok" } else { "none" }.to_string(),
        details: Some(format!("{} installed", hooks_count)),
    });
    if !json_output {
        println!("  {} Hooks ({} installed)",
            if hooks_count > 0 { "✓" } else { "!" },
            hooks_count);
    }

    // Summary
    let all_ok = am_exists && db_exists && qdrant_running && has_openai;

    if json_output {
        println!("{}", serde_json::json!({
            "healthy": all_ok,
            "checks": checks
        }));
    } else {
        println!();
        if all_ok {
            println!("  Status: All systems operational");
        } else {
            println!("  Status: Some components need attention");
            println!();
            if !docker_installed {
                println!("  Fix: Install Docker from https://docker.com");
            }
            if docker_installed && !docker_running {
                println!("  Fix: Start Docker Desktop or docker daemon");
            }
            if docker_running && !qdrant_running {
                println!("  Fix: Run 'docker start agentmem-qdrant' or 'am init'");
            }
            if !has_openai {
                println!("  Fix: Set OPENAI_API_KEY or run 'am init'");
            }
            if !am_exists {
                println!("  Fix: Run 'am init' in your project directory");
            }
        }
        println!();
    }

    Ok(())
}

/// Sync a memory to the cloud API
async fn sync_memory_to_cloud(memory_type: &str, title: &str, content: Option<&str>) -> Result<()> {
    use crate::api::{ApiClient, CreateMemoryRequest, Memory, get_machine_id};

    let client = ApiClient::new()?;

    // Get project name from current directory
    let project_name = std::env::current_dir()?
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let request = CreateMemoryRequest {
        project_name,
        session_id: None,
        scope: "project".to_string(),
        memory_type: memory_type.to_string(),
        title: title.to_string(),
        content: content.map(|s| s.to_string()),
        agent: Some("cli".to_string()),
        model: None,
        confidence: Some(80),
        machine_id: Some(get_machine_id()),
    };

    client.post::<Memory, _>("/api/memories", request).await?;
    Ok(())
}

/// List memories from cloud
async fn list_cloud_memories() -> Result<Vec<crate::api::Memory>> {
    use crate::api::ApiClient;

    let client = ApiClient::new()?;

    // Get project name from current directory
    let project_name = std::env::current_dir()?
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let path = format!("/api/memories?projectName={}", urlencoding::encode(&project_name));
    let memories: Vec<crate::api::Memory> = client.get(&path).await?;
    Ok(memories)
}

/// Run authentication commands
async fn run_auth(command: AuthCommands) -> Result<()> {
    use crate::api::{
        ApiClient, RegisterRequest,
        save_credentials, clear_credentials, get_api_credentials,
        prompt_api_key, prompt_email, get_machine_id,
    };

    match command {
        AuthCommands::Login { api_key } => {
            // Get API key from arg or prompt
            let key = match api_key {
                Some(k) => k,
                None => prompt_api_key()?,
            };

            // Verify the key works
            println!("Verifying API key...");
            let client = ApiClient::new()?.with_api_key(key.clone());

            match client.get::<crate::api::UserWithStats>("/api/auth/me").await {
                Ok(user) => {
                    // Save credentials
                    save_credentials(&key, &user.id, &user.email)?;

                    println!();
                    println!("✓ Logged in as {} ({})", user.email, user.name.unwrap_or_default());
                    println!();
                    println!("  Projects: {}", user.stats.projects);
                    println!("  Memories: {}", user.stats.memories);
                    println!("  Sessions: {}", user.stats.sessions);
                    println!();
                    println!("Credentials saved to ~/.agentmem/credentials");
                }
                Err(e) => {
                    anyhow::bail!("Login failed: {}. Check your API key.", e);
                }
            }
        }
        AuthCommands::Register { email, name } => {
            // Get email from arg or prompt
            let user_email = match email {
                Some(e) => e,
                None => prompt_email()?,
            };

            // Get name (optional)
            let user_name = name;

            println!("Creating account...");
            let client = ApiClient::new()?;

            let request = RegisterRequest {
                email: user_email.clone(),
                name: user_name,
            };

            match client.post::<crate::api::User, _>("/api/auth/register", request).await {
                Ok(user) => {
                    // Save credentials
                    if let Some(ref api_key) = user.api_key {
                        save_credentials(api_key, &user.id, &user.email)?;
                    }

                    println!();
                    println!("✓ Account created successfully!");
                    println!();
                    println!("  Email: {}", user.email);
                    if let Some(ref key) = user.api_key {
                        println!("  API Key: {}", key);
                        println!();
                        println!("  IMPORTANT: Save your API key - it won't be shown again!");
                    }
                    println!();
                    println!("Credentials saved to ~/.agentmem/credentials");
                }
                Err(e) => {
                    anyhow::bail!("Registration failed: {}", e);
                }
            }
        }
        AuthCommands::Status => {
            match get_api_credentials()? {
                Some(key) => {
                    println!("Checking authentication...");
                    let client = ApiClient::new()?.with_api_key(key);

                    match client.get::<crate::api::UserWithStats>("/api/auth/me").await {
                        Ok(user) => {
                            println!();
                            println!("✓ Authenticated");
                            println!();
                            println!("  Email: {}", user.email);
                            if let Some(name) = user.name {
                                println!("  Name: {}", name);
                            }
                            println!("  Machine: {}", get_machine_id());
                            println!();
                            println!("  Projects: {}", user.stats.projects);
                            println!("  Memories: {}", user.stats.memories);
                            println!("  Sessions: {}", user.stats.sessions);
                        }
                        Err(_) => {
                            println!();
                            println!("✗ API key is invalid or expired");
                            println!();
                            println!("Run 'am auth login' to re-authenticate.");
                        }
                    }
                }
                None => {
                    println!();
                    println!("✗ Not authenticated");
                    println!();
                    println!("Run 'am auth login' to authenticate.");
                    println!("Run 'am auth register' to create an account.");
                }
            }
        }
        AuthCommands::Logout => {
            clear_credentials()?;
            println!("✓ Logged out. Credentials cleared.");
        }
    }

    Ok(())
}

/// Run session commands (for cloud tracking)
async fn run_session(command: SessionCommands) -> Result<()> {
    use crate::api::{ApiClient, CreateSessionRequest, UpdateSessionRequest, Session, get_api_credentials, get_machine_id};

    // Handle local-only commands first (don't require cloud auth)
    match &command {
        SessionCommands::List { limit, json } => {
            let db_path = get_db_path();
            if db_path.exists() {
                let conn = get_connection(db_path)?;
                let sessions = crate::sessions::service::list_sessions(&conn, *limit)?;
                if *json {
                    println!("{}", serde_json::to_string_pretty(&sessions)?);
                } else {
                    if sessions.is_empty() {
                        println!("No sessions recorded.");
                    } else {
                        for s in sessions {
                            println!("{}: {} ({}) - {}",
                                s.id,
                                s.agent.clone().unwrap_or_else(|| "unknown".to_string()),
                                s.status,
                                s.started_at.format("%Y-%m-%d %H:%M")
                            );
                        }
                    }
                }
            }
            return Ok(());
        }
        SessionCommands::SaveTodos { snapshot_json } => {
            let db_path = get_db_path();
            if db_path.exists() {
                let conn = get_connection(db_path)?;
                // Get or create active session
                let session_id = match crate::sessions::service::get_active_session(&conn)? {
                    Some(s) => s.id,
                    None => crate::sessions::service::start_session(&conn, Some("claude-code"), None)?,
                };
                let snap_id = crate::sessions::service::save_todowrite_snapshot(&conn, &session_id, snapshot_json)?;
                println!("Saved TodoWrite snapshot: {}", snap_id);
            }
            return Ok(());
        }
        SessionCommands::GetTodos { json } => {
            let db_path = get_db_path();
            if db_path.exists() {
                let conn = get_connection(db_path)?;
                if let Some(snapshot) = crate::sessions::service::get_most_recent_snapshot(&conn)? {
                    if *json {
                        println!("{}", serde_json::to_string_pretty(&snapshot)?);
                    } else {
                        println!("{}", snapshot.snapshot_json);
                    }
                } else {
                    println!("No TodoWrite snapshot found.");
                }
            }
            return Ok(());
        }
        SessionCommands::Start { agent, model } => {
            // Start local session first
            let db_path = get_db_path();
            if db_path.exists() {
                let conn = get_connection(db_path)?;
                // Check if there's already an active session
                if let Some(existing) = crate::sessions::service::get_active_session(&conn)? {
                    // Reuse existing session
                    println!("Session active: {}", existing.id);
                    return Ok(());
                }
                // Create new local session
                let session_id = crate::sessions::service::start_session(
                    &conn,
                    agent.as_deref(),
                    model.as_deref(),
                )?;
                println!("Session started: {}", session_id);
            }
            return Ok(()); // Local-only for now
        }
        SessionCommands::End { .. } => {
            // End local session first
            let db_path = get_db_path();
            if db_path.exists() {
                let conn = get_connection(db_path)?;
                if let Some(session) = crate::sessions::service::get_active_session(&conn)? {
                    crate::sessions::service::end_session(&conn, &session.id, None)?;
                    println!("Session ended: {}", session.id);
                }
            }
            // Continue to cloud session end if authenticated
        }
        _ => {} // Continue to cloud commands
    }

    // Cloud commands require authentication
    if get_api_credentials()?.is_none() {
        // Silently skip if not authenticated (hooks should not fail)
        return Ok(());
    }

    let client = ApiClient::new()?;

    // Get project name from current directory
    let project_name = std::env::current_dir()?
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Session ID file for tracking current session
    let session_file = crate::config::get_agentmem_dir().join(".current_session");

    match command {
        SessionCommands::Start { agent, model } => {
            let request = CreateSessionRequest {
                project_name,
                agent: agent.unwrap_or_else(|| "unknown".to_string()),
                model,
                machine_id: Some(get_machine_id()),
            };

            match client.post::<Session, _>("/api/sessions", request).await {
                Ok(session) => {
                    // Save session ID for later
                    if let Err(e) = std::fs::write(&session_file, &session.id) {
                        eprintln!("Warning: Could not save session ID: {}", e);
                    }
                    println!("Session started: {}", session.id);
                }
                Err(e) => {
                    eprintln!("Warning: Could not start cloud session: {}", e);
                }
            }
        }
        SessionCommands::End { tokens_in, tokens_out, model } => {
            // Read session ID
            if let Ok(session_id) = std::fs::read_to_string(&session_file) {
                let session_id = session_id.trim();
                if !session_id.is_empty() {
                    let request = UpdateSessionRequest {
                        tokens_in,
                        tokens_out,
                        model,
                        end: Some(true),
                    };

                    let path = format!("/api/sessions/{}", session_id);
                    match client.put::<Session, _>(&path, request).await {
                        Ok(_) => {
                            println!("Session ended");
                            // Clean up session file
                            let _ = std::fs::remove_file(&session_file);
                        }
                        Err(e) => {
                            eprintln!("Warning: Could not end cloud session: {}", e);
                        }
                    }
                }
            }
        }
        SessionCommands::Status => {
            if let Ok(session_id) = std::fs::read_to_string(&session_file) {
                let session_id = session_id.trim();
                if !session_id.is_empty() {
                    let path = format!("/api/sessions/{}", session_id);
                    match client.get::<Session>(&path).await {
                        Ok(session) => {
                            println!("Current session: {}", session.id);
                            println!("  Agent: {}", session.agent);
                            if let Some(m) = session.model {
                                println!("  Model: {}", m);
                            }
                            println!("  Started: {}", session.started_at);
                            println!("  Tokens: {} in / {} out", session.tokens_in, session.tokens_out);
                        }
                        Err(_) => {
                            println!("No active session");
                        }
                    }
                } else {
                    println!("No active session");
                }
            } else {
                println!("No active session");
            }
        }
        // Local commands already handled above
        SessionCommands::List { .. } | SessionCommands::SaveTodos { .. } | SessionCommands::GetTodos { .. } => {
            unreachable!("Local commands handled above");
        }
    }

    Ok(())
}

/// Run plan commands
async fn run_plan(command: PlanCommands) -> Result<()> {
    let db_path = get_db_path();
    if !db_path.exists() {
        anyhow::bail!("AgentMem not initialized. Run 'am init' first.");
    }
    let conn = get_connection(db_path)?;

    match command {
        PlanCommands::Create { title, content, file } => {
            let id = crate::plans::service::create_plan(&conn, &title, content.as_deref(), file.as_deref())?;
            println!("Created plan: {} \"{}\"", id, title);
        }
        PlanCommands::List { status, json } => {
            let plans = crate::plans::service::list_plans(&conn, status.as_deref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&plans)?);
            } else {
                if plans.is_empty() {
                    println!("No plans found.");
                } else {
                    for p in plans {
                        println!("[{}] {}: {}", p.status, p.id, p.title);
                    }
                }
            }
        }
        PlanCommands::Show { id, json } => {
            if let Some(plan) = crate::plans::service::get_plan(&conn, &id)? {
                if json {
                    println!("{}", serde_json::to_string_pretty(&plan)?);
                } else {
                    println!("Plan: {} ({})", plan.title, plan.status);
                    println!("ID: {}", plan.id);
                    if let Some(f) = plan.file_path {
                        println!("File: {}", f);
                    }
                    if let Some(c) = plan.content {
                        println!("\n{}", c);
                    }
                    // Show linked tasks
                    let tasks = crate::plans::service::get_plan_tasks(&conn, &id)?;
                    if !tasks.is_empty() {
                        println!("\nLinked tasks:");
                        for t in tasks {
                            println!("  {} (order: {})", t.task_id, t.task_order);
                        }
                    }
                }
            } else {
                println!("Plan not found: {}", id);
            }
        }
        PlanCommands::Complete { id } => {
            crate::plans::service::complete_plan(&conn, &id)?;
            println!("Completed plan: {}", id);
        }
        PlanCommands::Abandon { id } => {
            crate::plans::service::abandon_plan(&conn, &id)?;
            println!("Abandoned plan: {}", id);
        }
        PlanCommands::Link { plan_id, task_id, order } => {
            crate::plans::service::link_task_to_plan(&conn, &plan_id, &task_id, order)?;
            println!("Linked task {} to plan {} (order: {})", task_id, plan_id, order);
        }
        PlanCommands::Active { json } => {
            if let Some(plan) = crate::plans::service::get_active_plan(&conn)? {
                if json {
                    println!("{}", serde_json::to_string_pretty(&plan)?);
                } else {
                    println!("Active plan: {} \"{}\"", plan.id, plan.title);
                    if let Some(f) = plan.file_path {
                        println!("  File: {}", f);
                    }
                }
            } else {
                println!("No active plan.");
            }
        }
        PlanCommands::ExtractTasks { file, plan_id, model, dry_run, json } => {
            // Read plan file
            let content = std::fs::read_to_string(&file)
                .context(format!("Failed to read plan file: {}", file))?;

            // Extract tasks using LLM
            let result = crate::plans::service::extract_tasks_from_plan(&content, &model).await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
                return Ok(());
            }

            if result.tasks.is_empty() {
                println!("No tasks extracted from plan.");
                return Ok(());
            }

            println!("Extracted {} tasks:", result.tasks.len());
            for task in &result.tasks {
                println!("  [P{}] {}: {}", task.priority, task.order, task.title);
            }

            if dry_run {
                println!("\n(Dry run - no tasks created)");
                return Ok(());
            }

            // Get plan ID to link to
            let link_plan_id = match plan_id {
                Some(id) => Some(id),
                None => crate::plans::service::get_active_plan(&conn)?
                    .map(|p| p.id),
            };

            // Create tasks
            println!("\nCreating tasks...");
            for task in &result.tasks {
                let task_id = crate::tasks::service::create_task(
                    &conn,
                    &task.title,
                    Some(&task.description),
                    task.priority,
                    "implementation",
                )?;
                println!("  Created: {} \"{}\"", task_id, task.title);

                // Link to plan if we have one
                if let Some(ref pid) = link_plan_id {
                    crate::plans::service::link_task_to_plan(&conn, pid, &task_id, task.order)?;
                }
            }

            if let Some(pid) = link_plan_id {
                println!("\nLinked to plan: {}", pid);
            }
        }
    }

    Ok(())
}
