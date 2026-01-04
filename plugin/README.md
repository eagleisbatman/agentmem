# AgentMem Plugin for Claude Code

Persistent memory for Claude Code - survives sessions and compaction.

## Quick Start

```bash
# 1. Install the CLI
cd /path/to/agentmem
cargo build --release
cp target/release/am ~/.local/bin/

# 2. Initialize in your project
cd your-project
am init

# 3. Start using Claude Code - context is auto-injected!
```

## What Works Automatically

| Feature | When It Happens |
|---------|-----------------|
| Context injection | Start of each prompt (memories, tasks, protected files) |
| Memory prompts | After git commits and successful builds |
| Session cleanup | When session ends |

## What Needs Manual Commands

### Save Memories
```bash
am mem add <type> <title> --content "<details>"
```

Or use the slash command:
```
/agentmem:remember decision "Use PostgreSQL for the database"
```

### After Creating a Plan
```bash
# Record the plan
am plan create "Feature Implementation" --file path/to/plan.md

# Extract tasks from it (uses GPT-4o)
am plan extract-tasks --file path/to/plan.md
```

### Save TodoWrite State (before ending session)
```bash
# Copy the JSON from your current todo list
am session save-todos '[{"content":"Task 1","status":"completed","activeForm":"..."}]'
```

### Protect Files
```bash
am protect src/auth/core.ts --reason "Critical auth logic"
```
Or: `/agentmem:protect src/auth/core.ts`

## Slash Commands

| Command | Description |
|---------|-------------|
| `/agentmem:remember <type> <title>` | Add a memory |
| `/agentmem:protect <file>` | Mark file as protected |
| `/agentmem:sync` | Export and push to git |
| `/agentmem:context` | Show current context |
| `/agentmem:status` | Check system health |

## CLI Quick Reference

```bash
# Context & Status
am context              # See injected context (memories, tasks, files)
am doctor               # Check system health

# Memories
am mem add <type> <title> --content "<details>"
am mem list             # List all memories
am mem search "query"   # Semantic search (requires Qdrant)

# Tasks
am task list            # List all tasks
am task create "Title"  # Create a task
am task update <id> <status>  # Update status

# Plans
am plan active          # See current plan
am plan list            # List all plans
am plan create "Title" --file plan.md
am plan extract-tasks --file plan.md  # Extract tasks via GPT-4o
am plan complete <id>   # Mark plan done

# Sessions
am session status       # Current session
am session save-todos '<json>'  # Save TodoWrite state
am session get-todos    # Restore last TodoWrite state

# Sync
am sync                 # Export to JSONL and git push
am sync --pull          # Pull and import from git
```

## Memory Types

| Type | Use For | Example |
|------|---------|---------|
| `decision` | Architectural choices | "Use React Query for data fetching" |
| `correction` | User corrections | "Don't use deprecated API" |
| `gotcha` | Things that broke | "Build fails without NODE_ENV" |
| `pattern` | Repeated behaviors | "Always run tests before commit" |
| `infrastructure` | URLs, configs | "API endpoint: api.example.com" |
| `tool` | Scripts, utilities | "Use scripts/deploy.sh for deploys" |
| `protected` | Files not to modify | "Don't touch auth/core.ts" |
| `insight` | Non-obvious discoveries | "Performance bottleneck in query X" |

## Skills (Auto-Invoked)

**memory-persistence**: Prompts you to save learnings when:
- You make architectural decisions
- User corrects you
- You encounter gotchas
- You discover infrastructure details

**plan-to-tasks**: Reminds you to extract tasks after creating a plan.

## Workflow Example

```bash
# Start of day - see what you were working on
am context
am task list

# After making a decision
am mem add decision "Use JWT for auth" --content "Chose JWT over sessions for stateless API"

# After creating a plan in Claude
am plan create "Auth System" --file ~/.claude/plans/auth-plan.md
am plan extract-tasks --file ~/.claude/plans/auth-plan.md

# Before ending session - save your todos
am session save-todos '[...your current todos...]'

# End of day - sync everything
am sync
```

## Troubleshooting

### Check if AgentMem is initialized
```bash
ls -la .agentmem/
am doctor
```

### View what context Claude sees
```bash
am context --format markdown
```

### Rebuild and reinstall CLI
```bash
cd /path/to/agentmem
cargo build --release
cp target/release/am ~/.local/bin/
```

### Check plugin is loaded
Look for `agentmem@local` in Claude Code's plugin list.
