# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AgentMem is an agent memory system for persistent context in AI coding agents. It provides a CLI tool (`am`) for managing tasks, memories, protected files, and tools, with SQLite storage and git-based synchronization.

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
```

## CLI Usage

The binary is named `am`. Initialize in a project first:

```bash
am init                              # Initialize .agentmem/ directory
am task create "Task title"          # Create a task
am task list                         # List all tasks
am mem add <type> <title>            # Add a memory
am context --query "search term"     # Retrieve relevant context
am sync --push                       # Export and push via git
```

## Architecture

```
src/
├── main.rs           # CLI entry point using clap, routes to module handlers
├── init.rs           # Project initialization (creates .agentmem/, database, config)
├── config/           # Configuration management (config.yaml parsing, paths)
├── db/               # SQLite layer
│   ├── sqlite.rs     # Connection management
│   ├── migrations.rs # Schema (10 tables: tasks, memories, embeddings, entities, etc.)
│   └── models.rs     # Data structures (Task, Memory, ProtectedFile, Tool, Entity)
├── memory/           # Memory storage
│   └── service.rs    # add_memory, list_memories, add_protected_file, add_tool
├── retrieval/        # Context retrieval for prompts
│   └── context.rs    # get_context, format_context_markdown
├── tasks/            # Task management
│   └── service.rs    # create_task, list_tasks, get_ready_tasks
├── sync/             # Git integration and data export/import
│   ├── git.rs        # Git add/commit/push
│   ├── export.rs     # Export to JSONL
│   └── import.rs     # Import from JSONL with conflict resolution
├── hooks/            # Hook system for AI agent integration
│   ├── service.rs    # install_hooks, list_hooks, test_hook
│   └── templates.rs  # Embedded JS hook templates (pre-prompt, post-session)
└── embedding/        # Pluggable embedding providers (stubs for openai, gemini, ollama)
```

## Key Patterns

- **Error handling**: Uses `anyhow::Result` throughout
- **Database**: All persistent state in SQLite at `.agentmem/agentmem.db`
- **Sync format**: JSONL files for git-friendly line-delimited JSON
- **Configuration**: YAML config at `.agentmem/config.yaml`

## Data Flow

1. CLI command parsed by clap in `main.rs`
2. Routes to appropriate module handler
3. Handler gets SQLite connection via `db::get_connection()`
4. Operates on database, returns data
5. `main.rs` formats output (JSON or plain text)

## Current Limitations

- Memory search uses LIKE-based queries (semantic search stubs exist but aren't implemented)
- Embedding providers are defined but minimal
- Memory extraction from transcripts is a stub

## AgentMem Integration

This project uses AgentMem for persistent context across sessions.

**Quick commands:**
- `am task ready` - See unblocked tasks
- `am context` - Get relevant memories
- `am mem add <type> <title>` - Add a memory
- `am protect <file>` - Mark file as protected
- `am sync` - Sync to git

Protected files require approval before modification.
