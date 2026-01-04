# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AgentMem is an agent memory system for persistent context in AI coding agents. It provides:
- A CLI tool (`am`) for managing tasks, memories, protected files, and tools
- A Claude Code plugin for automatic integration
- SQLite storage with git-based synchronization
- Optional semantic search via OpenAI embeddings and Qdrant

## Build & Development Commands

```bash
make build          # Build release binary (cargo build --release)
make install        # Install to system via cargo
make test           # Run tests (cargo test)
make clean          # Clean build artifacts

# Run single test
cargo test <test_name>

# Run with verbose output
cargo test -- --nocapture

# Run MCP server (for plugin)
cargo run -- mcp-server
```

## CLI Usage

The binary is named `am`. Initialize in a project first:

```bash
am init                              # Initialize .agentmem/ + install plugin
am task create "Task title"          # Create a task
am task update am-xxxx in_progress   # Update task status
am task history am-xxxx              # View task history
am mem add <type> <title>            # Add a memory
am plan create "Plan title"          # Create a plan
am context --query "search term"     # Retrieve relevant context
am sync --push                       # Export and push via git
```

## Architecture

```
src/
├── main.rs           # CLI entry point using clap, routes to module handlers
├── init.rs           # Project initialization (creates .agentmem/, plugin install)
├── config/           # Configuration management
│   └── config.rs     # config.yaml parsing, hierarchical discovery, project IDs
├── db/               # SQLite layer
│   ├── sqlite.rs     # Connection management
│   ├── migrations.rs # Schema (15 tables including plans, sessions, task_history)
│   └── models.rs     # Data structures (Task, Memory, Plan, Session, etc.)
├── memory/           # Memory storage
│   ├── service.rs    # add_memory, list_memories, add_protected_file
│   └── extraction.rs # GPT-4o based memory extraction from transcripts
├── retrieval/        # Context retrieval for prompts
│   ├── context.rs    # get_context, format_context_markdown
│   └── search.rs     # Semantic search via Qdrant
├── tasks/            # Task management
│   └── service.rs    # create_task, update_task_status, get_task_history
├── plans/            # Plan management (new in 2.0)
│   └── service.rs    # create_plan, complete_plan, link_task_to_plan
├── sessions/         # Session management (new in 2.0)
│   └── service.rs    # start_session, save_todowrite_snapshot
├── sync/             # Git integration and data export/import
│   ├── git.rs        # Git add/commit/push
│   ├── export.rs     # Export to JSONL
│   └── import.rs     # Import from JSONL with conflict resolution
├── hooks/            # Legacy hook system (deprecated in 2.0)
│   ├── service.rs    # install_hooks (now shows deprecation notice)
│   └── templates.rs  # JS hook templates for legacy support
├── mcp/              # MCP server for Claude Code plugin (new in 2.0)
│   └── server.rs     # JSON-RPC protocol, 10 tools exposed
└── embedding/        # Pluggable embedding providers
    ├── service.rs    # Provider trait and factory
    ├── openai.rs     # OpenAI embeddings (text-embedding-3-small)
    └── qdrant.rs     # Qdrant vector store
```

## Plugin Structure

```
plugin/
├── .claude-plugin/
│   └── plugin.json       # Plugin manifest
├── commands/             # Slash commands
│   ├── remember.md       # /agentmem:remember <type> <title>
│   ├── protect.md        # /agentmem:protect <file>
│   ├── context.md        # /agentmem:context
│   ├── sync.md           # /agentmem:sync
│   └── status.md         # /agentmem:status
├── skills/               # Auto-invoked skills
│   ├── memory-persistence/
│   │   └── SKILL.md      # Detect and save learnings
│   └── plan-to-tasks/
│       └── SKILL.md      # Convert plans to tasks
├── hooks/
│   └── hooks.json        # UserPromptSubmit, Stop hooks
└── .mcp.json             # MCP server config
```

## Key Patterns

- **Error handling**: Uses `anyhow::Result` throughout
- **Database**: All persistent state in SQLite at `.agentmem/agentmem.db`
- **Sync format**: JSONL files for git-friendly line-delimited JSON
- **Configuration**: YAML config at `.agentmem/config.yaml`
- **Hierarchical discovery**: `find_agentmem_dir()` walks up tree to find `.agentmem/`
- **Project isolation**: Unique Qdrant collection per project via `get_project_id()`

## Data Flow

1. CLI command parsed by clap in `main.rs`
2. Routes to appropriate module handler
3. Handler uses hierarchical discovery to find `.agentmem/`
4. Gets SQLite connection via `db::get_connection()`
5. Operates on database, returns data
6. `main.rs` formats output (JSON or plain text)

## Database Schema (15 tables)

Core tables:
- `tasks` - Task tracking with status, priority, type
- `memories` - 8 memory types with confidence scores
- `protected_files` - Files not to modify
- `tools` - Registered scripts/utilities

New in 2.0:
- `plans` - Plan records with content and status
- `plan_tasks` - Links tasks to plans with ordering
- `sessions` - Session tracking with start/end times
- `todowrite_snapshots` - Persisted TodoWrite state
- `task_history` - Audit trail of status changes

Search/embedding:
- `memory_embeddings` - Vector storage (BLOB)
- `entities` - Named entity extraction
- `memory_entities` - Entity-memory links
- `session_recalls` - Memory recall tracking
- `config` - Key-value settings

## AgentMem Integration

This project uses AgentMem for persistent context across sessions.

**Quick commands:**
- `am task ready` - See unblocked tasks
- `am context` - Get relevant memories
- `am mem add <type> <title>` - Add a memory
- `am protect <file>` - Mark file as protected
- `am sync` - Sync to git

**Plugin commands (in Claude Code):**
- `/agentmem:remember <type> <title>` - Add a memory
- `/agentmem:protect <file>` - Protect a file
- `/agentmem:context` - Show current context
- `/agentmem:sync` - Sync to git

Protected files require approval before modification.
