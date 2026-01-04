# AgentMem

**Persistent memory for AI coding agents.** Stop repeating yourself to Claude, Cursor, and other AI assistants.

AgentMem solves the "agent amnesia" problem - AI coding agents forget everything between sessions. They recreate existing scripts, modify files they shouldn't touch, and repeat the same mistakes. AgentMem gives them persistent memory.

## What's New in 2.0

- **Claude Code Plugin** - Native integration via skills and slash commands
- **Automatic Persistence** - TodoWrite state, plans, and learnings survive sessions
- **Workspace Support** - Hierarchical `.agentmem/` for monorepos
- **Project Isolation** - Per-project Qdrant collections

## How It Works

```
┌─────────────────────────────────────────────────────────────┐
│  Your prompt: "Fix the auth bug"                            │
│                      │                                      │
│                      ▼                                      │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ AgentMem injects relevant context:                      ││
│  │                                                         ││
│  │ ## Protected Files (don't modify without asking)        ││
│  │ - src/prompts/system.md                                 ││
│  │                                                         ││
│  │ ## Relevant Memories                                    ││
│  │ - [decision] Use JWT for auth tokens                    ││
│  │ - [gotcha] Session cookie breaks on Safari              ││
│  │                                                         ││
│  │ ## Available Tools                                      ││
│  │ - scripts/auth-debug.sh - Debug auth flow               ││
│  └─────────────────────────────────────────────────────────┘│
│                      │                                      │
│                      ▼                                      │
│  Agent works with full context, avoids past mistakes        │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

```bash
# 1. Install
cargo install --git https://github.com/eagleisbatman/agentmem

# 2. Initialize in your project (installs Claude Code plugin automatically)
cd your-project
am init

# 3. Start using Claude Code - context is now automatic!
```

## Installation

### From Source (Rust)

```bash
# Clone and build
git clone https://github.com/eagleisbatman/agentmem
cd agentmem
cargo build --release

# Add to PATH
cp target/release/am ~/.local/bin/am
```

### Requirements

- **Rust** 1.70+ (for building)
- **Docker** (optional, for semantic search with Qdrant)
- **OpenAI API key** (optional, for embeddings and memory extraction)

## Claude Code Plugin

AgentMem 2.0 installs as a Claude Code plugin with slash commands and automatic hooks.

### Plugin Commands

| Command | Description |
|---------|-------------|
| `/agentmem:remember <type> <title>` | Add a memory |
| `/agentmem:protect <file>` | Protect a file from modification |
| `/agentmem:context` | Show current context |
| `/agentmem:sync` | Sync to git |
| `/agentmem:status` | Check system health |

### Automatic Features

- **Pre-prompt hook**: Injects relevant context before each prompt
- **Session end hook**: Syncs data when sessions end
- **Memory persistence skill**: Auto-detects learnings to save
- **Plan-to-tasks skill**: Converts plans to tracked tasks

### Manual Plugin Installation

If `am init` didn't install the plugin:

```bash
# Copy plugin to Claude Code plugins directory
cp -r /path/to/agentmem/plugin ~/.claude/plugins/agentmem
```

## Usage

### Initialize a Project

```bash
am init
```

Creates `.agentmem/` directory with SQLite database, config, and installs the Claude Code plugin.

### Protect Important Files

Prevent the AI from modifying critical files without asking:

```bash
am protect src/prompts/system.md "Production prompt - don't change"
am protect prisma/schema.prisma "Database schema"
am protect "*.env*" "Environment secrets"
```

### Add Memories

Teach the AI about your project:

```bash
# Record a decision
am mem add decision "Use PostgreSQL" --content "Chose Postgres for JSON support"

# Record a gotcha (things that break)
am mem add gotcha "Safari cookie issue" --content "SameSite=Lax breaks auth on Safari"

# Record infrastructure details
am mem add infrastructure "API endpoint" --content "https://api.myapp.com"

# Register existing tools/scripts
am tool scripts/deploy.sh "Deploy to production" "Usage: ./scripts/deploy.sh staging|prod"
```

### Track Tasks

```bash
# Create tasks
am task create "Fix authentication bug" --priority 1 --type bug
am task create "Add OAuth login" --priority 2

# Update task status
am task update am-xxxx in_progress

# See task history
am task history am-xxxx

# List all tasks
am task list
```

### Manage Plans

```bash
# Create a plan
am plan create "Implement OAuth" --content "1. Add OAuth provider..."

# Link tasks to plan
am plan link plan-xxxx am-yyyy

# Show plan with tasks
am plan show plan-xxxx

# Complete a plan
am plan complete plan-xxxx
```

### Get Context (What the Agent Sees)

```bash
# See what context would be injected
am context --query "fix auth bug"

# JSON format for debugging
am context --query "fix auth bug" --json
```

### Sync to Git

```bash
# Export and commit to git
am sync

# Also push to remote
am sync --push
```

## Memory Types

| Type | Purpose | Example |
|------|---------|---------|
| `decision` | Architectural choices | "Use Prisma ORM for type safety" |
| `correction` | When you corrected the AI | "Use tabs not spaces" |
| `gotcha` | Things that broke | "Raw SQL breaks Prisma migrations" |
| `pattern` | Repeated preferences | "Always run tests before committing" |
| `infrastructure` | URLs, endpoints, configs | "Production API at https://..." |
| `protected` | Files not to modify | "Don't touch system.md" |
| `tool` | Existing scripts/utilities | "Use scripts/translate.ts for i18n" |
| `insight` | Non-obvious discoveries | "Viettel AI better for Vietnamese" |

## Semantic Search (Optional)

Enable intelligent memory retrieval with OpenAI embeddings and Qdrant:

```bash
# Start Qdrant (Docker) - am init can do this automatically
docker run -d --name agentmem-qdrant \
  -p 6333:6333 -p 6334:6334 \
  -v agentmem-qdrant-data:/qdrant/storage \
  qdrant/qdrant

# Set OpenAI key (or enter during am init)
export OPENAI_API_KEY="sk-..."

# Initialize with embeddings
am init --embedding openai

# Now searches use semantic similarity
am mem search "authentication issues"
```

## Workspace Support

AgentMem supports monorepos with hierarchical `.agentmem/` directories:

```
my-monorepo/
├── .agentmem/           # Shared memories for all projects
├── packages/
│   ├── frontend/
│   │   └── .agentmem/   # Frontend-specific memories
│   └── backend/
│       └── .agentmem/   # Backend-specific memories
```

Running `am` from any subdirectory automatically finds the nearest `.agentmem/`.

## CLI Reference

### Core Commands

| Command | Description |
|---------|-------------|
| `am init` | Initialize AgentMem in current project |
| `am context` | Get context for a query |
| `am sync` | Export and commit to git |
| `am doctor` | Check system health |

### Memory Commands

| Command | Description |
|---------|-------------|
| `am mem add <type> <title>` | Add a memory |
| `am mem list` | List all memories |
| `am mem search <query>` | Semantic search memories |

### Task Commands

| Command | Description |
|---------|-------------|
| `am task create <title>` | Create a task |
| `am task list` | List all tasks |
| `am task ready` | Show unblocked tasks |
| `am task update <id> <status>` | Update task status |
| `am task history <id>` | Show task history |
| `am task show <id>` | Show task details |

### Plan Commands

| Command | Description |
|---------|-------------|
| `am plan create <title>` | Create a plan |
| `am plan list` | List all plans |
| `am plan show <id>` | Show plan with linked tasks |
| `am plan complete <id>` | Mark plan as complete |
| `am plan link <plan_id> <task_id>` | Link task to plan |
| `am plan active` | Show active plan |

### Quick Commands

| Command | Description |
|---------|-------------|
| `am protect <file> [reason]` | Mark file as protected |
| `am tool <path> <description>` | Register a script/utility |

### Session Commands

| Command | Description |
|---------|-------------|
| `am session start` | Start a new session |
| `am session end` | End current session |
| `am session list` | List sessions |
| `am session save-todos <json>` | Save TodoWrite state |
| `am session get-todos` | Get last TodoWrite snapshot |

### Legacy Hook Commands

| Command | Description |
|---------|-------------|
| `am hook install <agent>` | Install hooks (deprecated, use plugin) |
| `am hook list` | List installed hooks |

## Configuration

Config file: `.agentmem/config.yaml`

```yaml
embedding:
  provider: "openai"  # or: ollama, gemini, none
  model: "text-embedding-3-small"

qdrant:
  url: "http://localhost:6334"
  collection: "agentmem_myproject_abc123"  # Auto-generated per project

hooks:
  pre_prompt:
    enabled: true
    timeout_ms: 5000
  post_session:
    enabled: true
    auto_extract: true
```

## File Structure

```
your-project/
├── .agentmem/
│   ├── agentmem.db          # SQLite database (gitignored)
│   ├── agentmem.jsonl       # Git-synced data
│   ├── config.yaml          # Configuration
│   └── .gitignore           # Ignores db and hooks
├── .claude/
│   └── settings.json        # Your Claude Code settings
└── ~/.claude/plugins/
    └── agentmem/            # Claude Code plugin (installed by am init)
        ├── .claude-plugin/
        │   └── plugin.json
        ├── commands/        # Slash commands
        ├── skills/          # Auto-invoked skills
        ├── hooks/           # Hook definitions
        └── .mcp.json        # MCP server config
```

## Troubleshooting

### Check System Health

```bash
am doctor
```

Shows status of: database, Docker, Qdrant, OpenAI key, hooks, plugin.

### Plugin Not Working

1. Check plugin is installed: `ls ~/.claude/plugins/agentmem`
2. Reinstall: `cp -r plugin ~/.claude/plugins/agentmem`
3. Restart Claude Code

### Hooks Giving Errors

If you see errors about missing hook files:

1. The old hook format is deprecated
2. Remove old hooks: Edit `.claude/settings.json` and clear the `hooks` object
3. Use the plugin instead (installed to `~/.claude/plugins/agentmem`)

### Semantic Search Not Working

1. Check Qdrant is running: `docker ps | grep qdrant`
2. Check OpenAI key is set: `echo $OPENAI_API_KEY`
3. Re-run `am init --embedding openai`

### Command Not Found From Subdirectory

AgentMem uses hierarchical discovery - it walks up the directory tree to find `.agentmem/`. If `am` commands fail:

1. Make sure `.agentmem/` exists in current dir or a parent
2. Run `am init` if needed

## Development

```bash
# Build
make build        # or: cargo build --release

# Test
make test         # or: cargo test

# Run from source
cargo run -- init
cargo run -- mem add decision "Test"

# Run MCP server (for plugin)
cargo run -- mcp-server
```

## Architecture

See [CLAUDE.md](CLAUDE.md) for codebase architecture and development guidance.

## Upgrading from 1.x

See [docs/MIGRATION.md](docs/MIGRATION.md) for upgrade instructions.

## Roadmap

See [docs/ROADMAP.md](docs/ROADMAP.md) for planned features.

## License

MIT
