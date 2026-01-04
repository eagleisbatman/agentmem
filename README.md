# AgentMem

**Persistent memory for AI coding agents.** Stop repeating yourself to Claude, Cursor, and other AI assistants.

AgentMem solves the "agent amnesia" problem - AI coding agents forget everything between sessions. They recreate existing scripts, modify files they shouldn't touch, and repeat the same mistakes. AgentMem gives them persistent memory.

## How It Works

```
┌─────────────────────────────────────────────────────────────┐
│  Your prompt: "Fix the auth bug"                            │
│                      │                                      │
│                      ▼                                      │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ AgentMem injects relevant context:                   │   │
│  │                                                      │   │
│  │ ## Protected Files (don't modify without asking)     │   │
│  │ - src/prompts/system.md                              │   │
│  │                                                      │   │
│  │ ## Relevant Memories                                 │   │
│  │ - [decision] Use JWT for auth tokens                 │   │
│  │ - [gotcha] Session cookie breaks on Safari           │   │
│  │                                                      │   │
│  │ ## Available Tools                                   │   │
│  │ - scripts/auth-debug.sh - Debug auth flow            │   │
│  └─────────────────────────────────────────────────────┘   │
│                      │                                      │
│                      ▼                                      │
│  Agent works with full context, avoids past mistakes        │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

```bash
# 1. Install
cargo install --git https://github.com/your-username/agentmem

# 2. Initialize in your project
cd your-project
am init

# 3. Install hooks for your AI agent
am hook install claude-code    # or: gemini-cli, cursor

# 4. Start using your AI agent - context is now automatic!
```

## Installation

### From Source (Rust)

```bash
# Clone and build
git clone https://github.com/your-username/agentmem
cd agentmem
cargo build --release

# Add to PATH
cp target/release/am /usr/local/bin/am
```

### Requirements

- **Rust** 1.70+ (for building)
- **Docker** (optional, for semantic search with Qdrant)
- **OpenAI API key** (optional, for embeddings and memory extraction)

## Usage

### Initialize a Project

```bash
am init
```

Creates `.agentmem/` directory with SQLite database and config.

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

# See what's ready to work on
am task ready

# List all tasks
am task list
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
# Start Qdrant (Docker)
docker run -d -p 6333:6333 -p 6334:6334 qdrant/qdrant

# Set OpenAI key
export OPENAI_API_KEY="sk-..."
# Or save to ~/.agentmem/credentials

# Re-initialize with embeddings
am init --embedding openai

# Now searches use semantic similarity
am mem search "authentication issues"
```

## Agent Integration

### Claude Code

```bash
am hook install claude-code
```

This adds hooks to `.claude/settings.json` that:
1. **Pre-prompt**: Injects relevant context before each prompt
2. **Post-session**: Extracts memories from the session transcript

### Gemini CLI

```bash
am hook install gemini-cli
```

### Cursor

```bash
am hook install cursor
```

### Manual Integration

For unsupported agents, add to your agent's context file:

```markdown
## AgentMem

Before starting work:
1. Run `am context --query "<your task>"` to get relevant memories
2. Check for protected files before modifying anything

When you learn something:
- `am mem add <type> <title>` - Record the learning
- `am protect <file>` - Mark files as protected
```

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

### Quick Commands

| Command | Description |
|---------|-------------|
| `am protect <file> [reason]` | Mark file as protected |
| `am tool <path> <description>` | Register a script/utility |

### Hook Commands

| Command | Description |
|---------|-------------|
| `am hook install <agent>` | Install hooks for an agent |
| `am hook list` | List installed hooks |

### Sync Commands

| Command | Description |
|---------|-------------|
| `am sync` | Export and commit to git |
| `am export` | Export to JSONL file |
| `am import` | Import from JSONL file |

## Configuration

Config file: `.agentmem/config.yaml`

```yaml
embedding:
  provider: "openai"  # or: ollama, gemini, none
  model: "text-embedding-3-small"

qdrant:
  url: "http://localhost:6334"
  collection: "agentmem_memories"

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
│   └── hooks/               # Hook scripts
│       ├── pre-prompt.cjs
│       └── post-session.cjs
└── .claude/
    └── settings.json        # Updated with hook references
```

## Troubleshooting

### Check System Health

```bash
am doctor
```

Returns JSON with status of: database, Docker, Qdrant, OpenAI key, hooks.

### Hooks Not Working

1. Check hooks are installed: `am hook list`
2. Verify settings.json was updated
3. Test hook manually: Run `am context --query "test"`

### Semantic Search Not Working

1. Check Qdrant is running: `docker ps | grep qdrant`
2. Check OpenAI key is set: `echo $OPENAI_API_KEY`
3. Re-run `am init --embedding openai`

## Development

```bash
# Build
make build        # or: cargo build --release

# Test
make test         # or: cargo test

# Run from source
cargo run -- init
cargo run -- mem add decision "Test"
```

## Architecture

See [CLAUDE.md](CLAUDE.md) for codebase architecture and development guidance.

## Roadmap

See [docs/ROADMAP.md](docs/ROADMAP.md) for planned features.

## License

MIT
