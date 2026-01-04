# Migrating from AgentMem 1.x to 2.0

This guide helps you upgrade from AgentMem 1.x to the new 2.0 plugin-based architecture.

## What's Changed

### Plugin-Based Integration (Recommended)

AgentMem 2.0 introduces a Claude Code plugin that replaces the old hook-based integration:

| 1.x (Hooks) | 2.0 (Plugin) |
|-------------|--------------|
| `am hook install claude-code` | `am init` (auto-installs plugin) |
| `.agentmem/hooks/pre-prompt.js` | `~/.claude/plugins/agentmem/` |
| Manual hook scripts | Automatic via plugin |
| Per-project hooks | Single global plugin |

### New Features in 2.0

- **Plan Management**: `am plan create`, `am plan link`
- **Task History**: `am task update`, `am task history`
- **Session Tracking**: `am session start/end`, TodoWrite persistence
- **Workspace Support**: Hierarchical `.agentmem/` discovery
- **Project Isolation**: Per-project Qdrant collections
- **MCP Server**: `am mcp-server` for plugin communication

### Deprecated Features

- Hook installation via `am hook install` (still works but shows deprecation notice)
- Per-project hook scripts in `.agentmem/hooks/`
- Old hook format in `.claude/settings.json`

## Migration Steps

### Step 1: Update AgentMem

```bash
cd /path/to/agentmem
git pull
cargo build --release
cp target/release/am ~/.local/bin/am
```

### Step 2: Remove Old Hooks

Edit your project's `.claude/settings.json` and remove old hook configuration:

```json
{
  "hooks": {}
}
```

Or delete the hooks section entirely.

### Step 3: Install the Plugin

Run `am init` in your project to install the plugin:

```bash
cd your-project
am init
```

This will:
- Create/update `.agentmem/` directory
- Install plugin to `~/.claude/plugins/agentmem/`
- Set up project-specific Qdrant collection

### Step 4: Verify Installation

```bash
# Check plugin is installed
ls ~/.claude/plugins/agentmem

# Check system health
am doctor

# Test context retrieval
am context --query "test"
```

### Step 5: Clean Up Old Hook Files (Optional)

You can safely delete the old hook scripts:

```bash
rm -rf .agentmem/hooks/
```

## Database Migration

The database schema is automatically migrated when you run any `am` command. New tables are added:

- `plans` - Plan records
- `plan_tasks` - Task-plan links
- `sessions` - Session tracking
- `todowrite_snapshots` - TodoWrite state persistence
- `task_history` - Status change audit trail

Your existing data (tasks, memories, protected files, tools) is preserved.

## Breaking Changes

### Hook Format

**Before (1.x):**
```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "node .agentmem/hooks/pre-prompt.js"
          }
        ]
      }
    ]
  }
}
```

**After (2.0):**
Hooks are now defined in the plugin at `~/.claude/plugins/agentmem/hooks/hooks.json` and use the `am` CLI directly:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "am context --query \"$PROMPT\" --format markdown"
          }
        ]
      }
    ]
  }
}
```

### Subdirectory Support

In 1.x, hooks with relative paths broke when running from subdirectories. In 2.0:

- The plugin uses the `am` binary which handles hierarchical discovery
- `am` walks up the directory tree to find `.agentmem/`
- Works from any subdirectory within your project

## Troubleshooting

### "Cannot find module" Errors

If you see errors like:
```
Error: Cannot find module '/path/to/project/src/.agentmem/hooks/pre-prompt.js'
```

This means old hooks are still configured. Fix by:
1. Editing `.claude/settings.json`
2. Setting `"hooks": {}`
3. Restarting Claude Code

### Plugin Not Loading

1. Check plugin exists: `ls ~/.claude/plugins/agentmem`
2. Reinstall: `cp -r /path/to/agentmem/plugin ~/.claude/plugins/agentmem`
3. Restart Claude Code

### Commands Not Found

If `am` commands fail:
1. Ensure `am` is in PATH: `which am`
2. Rebuild and reinstall: `cargo build --release && cp target/release/am ~/.local/bin/`

## Rolling Back

If you need to rollback to 1.x behavior:

```bash
# Reinstall old hooks
am hook install claude-code

# Remove plugin
rm -rf ~/.claude/plugins/agentmem
```

However, we recommend staying on 2.0 for better stability and features.

## Getting Help

- Check `am doctor` for system health
- File issues at: https://github.com/eagleisbatman/agentmem/issues
- See full documentation in README.md
