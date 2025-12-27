pub mod config;
pub mod db;
pub mod embedding;
pub mod hooks;
pub mod init;
pub mod memory;
pub mod retrieval;
pub mod sync;
pub mod tasks;

use clap::{Parser, Subcommand};
use anyhow::Result;
use crate::init::run_init;
use crate::config::get_db_path;
use crate::db::get_connection;
use crate::tasks::service::{create_task, list_tasks, get_ready_tasks};
use crate::memory::service::{add_memory, add_memory_with_embedding, list_memories, add_protected_file, add_tool};
use crate::sync::{export_to_jsonl, import_from_jsonl, git_sync};
use crate::retrieval::context::{get_context, get_context_async, format_context_markdown};
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
}

#[derive(Subcommand)]
enum MemoryCommands {
    Add {
        memory_type: String,
        title: String,
        #[arg(short, long)]
        content: Option<String>,
    },
    List {
        #[arg(long)]
        json: bool,
    },
    Search {
        query: String,
    },
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
            }
        },
        Commands::Mem { command } => {
            let db_path = get_db_path();
            if !db_path.exists() {
                anyhow::bail!("AgentMem not initialized. Run 'am init' first.");
            }
            let conn = get_connection(db_path)?;
            match command {
                MemoryCommands::Add { memory_type, title, content } => {
                    let (id, embedded) = add_memory_with_embedding(&conn, &memory_type, &title, content.as_deref()).await?;
                    if embedded {
                        println!("✓ Added memory with embedding: {} \"{}\"", id, title);
                    } else {
                        println!("✓ Added memory: {} \"{}\"", id, title);
                    }
                },
                MemoryCommands::List { json } => {
                    let memories = list_memories(&conn)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&memories)?);
                    } else {
                        for m in memories {
                            println!("[{}] {}: {}", m.memory_type, m.id, m.title);
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
            git_sync(push, message.as_deref())?;
            println!("✓ Synced with git");
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
        Commands::Import { path } => {
            let db_path = get_db_path();
            let conn = get_connection(db_path)?;
            let import_path = path.unwrap_or_else(|| ".agentmem/agentmem.jsonl".to_string());
            import_from_jsonl(&conn, import_path)?;
            println!("✓ Imported from JSONL");
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
