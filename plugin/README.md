# AgentMem Plugin for Claude Code

Persistent memory for Claude Code - survives sessions and compaction.

## Installation

### Development (Local Testing)

```bash
claude --plugin-dir /path/to/agentmem/plugin
```

### Production (Global Install)

```bash
# Copy plugin to Claude's plugin directory
cp -r plugin ~/.claude/plugins/agentmem

# Install skills to user skills directory
mkdir -p ~/.claude/skills/agentmem-memory
mkdir -p ~/.claude/skills/agentmem-plan
cp plugin/skills/memory-persistence/SKILL.md ~/.claude/skills/agentmem-memory/SKILL.md
cp plugin/skills/plan-to-tasks/SKILL.md ~/.claude/skills/agentmem-plan/SKILL.md

# Update skill names to match directories
sed -i '' 's/name: memory-persistence/name: agentmem-memory/' ~/.claude/skills/agentmem-memory/SKILL.md
sed -i '' 's/name: plan-to-tasks/name: agentmem-plan/' ~/.claude/skills/agentmem-plan/SKILL.md
```

> **Note**: Skills must be installed to `~/.claude/skills/` (not the plugin directory) for Claude Code to load them.

## Prerequisites

1. **AgentMem CLI** must be installed and in PATH:
   ```bash
   cd /path/to/agentmem
   cargo build --release
   cp target/release/am ~/.local/bin/
   ```

2. **Initialize AgentMem** in your project:
   ```bash
   cd your-project
   am init
   ```

## Features

### Slash Commands

| Command | Description |
|---------|-------------|
| `/agentmem:remember <type> <title>` | Add a memory |
| `/agentmem:protect <file>` | Mark file as protected |
| `/agentmem:sync` | Sync to git |
| `/agentmem:context` | Show current context |
| `/agentmem:status` | Check system health |

### Skills (Auto-Invoked)

**memory-persistence**: Automatically saves learnings when you:
- Get corrected by the user
- Make decisions
- Encounter gotchas
- Discover infrastructure details

**plan-to-tasks**: Converts implementation plans into trackable tasks.

### Hooks

- **UserPromptSubmit**: Injects relevant context before each prompt
- **Stop**: Syncs data at session end

## Memory Types

| Type | Description |
|------|-------------|
| `decision` | Architectural/technical choices |
| `correction` | User corrections |
| `gotcha` | Things that broke |
| `pattern` | Repeated behaviors |
| `infrastructure` | URLs, endpoints, configs |
| `tool` | Scripts and utilities |
| `protected` | Files not to modify |
| `insight` | Non-obvious discoveries |

## How It Works

1. **Pre-prompt**: Context is injected showing protected files, tasks, and relevant memories
2. **During session**: Skills detect learnings and save them automatically
3. **Session end**: Data is synced to git for persistence

## Troubleshooting

Check status:
```bash
am doctor
```

View context:
```bash
am context --format markdown
```

List memories:
```bash
am mem list
```
