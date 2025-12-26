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
use crate::memory::service::{add_memory, list_memories, add_protected_file, add_tool};
use crate::sync::{export_to_jsonl, import_from_jsonl, git_sync};
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

fn main() -> Result<()> {
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
                    let id = add_memory(&conn, &memory_type, &title, content.as_deref())?;
                    println!("✓ Added memory: {} \"{}\"", id, title);
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
                MemoryCommands::Search { .. } => println!("Memory search not implemented yet (requires embeddings)"),
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
        Commands::Context { .. } => println!("Context not implemented yet"),
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
    }

    Ok(())
}
