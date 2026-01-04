# AgentMem 2.0 Design Document

> **Vision**: AgentMem as a persistence layer for Claude Code's native features, not a replacement.

**Date**: January 2025
**Status**: Design Complete, Ready for Implementation

---

## Problem Statement

Claude Code has powerful features that are **ephemeral**:

| Feature | What It Does | Problem |
|---------|--------------|---------|
| TodoWrite | Task tracking during session | Lost on session end/compaction |
| Plan mode | Structured planning before coding | Plans lost after compaction |
| Sub-agents | Spawn specialized workers | Each starts fresh, no shared memory |

**Result**: Developers repeat themselves, agents forget context, work is lost.

---

## Solution: Persistence Layer

AgentMem 2.0 works **with** Claude's native features, not against them.

```
┌─────────────────────────────────────────────────────────────────┐
│                    THE 2.0 APPROACH                             │
│                                                                 │
│  BEFORE (1.0):                                                  │
│  - AgentMem has own task system (am task)                       │
│  - Fights with Claude's TodoWrite                               │
│  - User must manually run commands                              │
│                                                                 │
│  AFTER (2.0):                                                   │
│  - Claude uses TodoWrite naturally                              │
│  - AgentMem persists TodoWrite state automatically              │
│  - Next session, state is restored                              │
│  - User does nothing, it just works                             │
└─────────────────────────────────────────────────────────────────┘
```

---

## Core Integrations

### 1. TodoWrite ↔ AgentMem Sync

```
During Session:
├── Claude uses TodoWrite normally
├── Post-response hook captures TodoWrite state
└── Saves to AgentMem database

Session End:
├── Final TodoWrite state saved
├── Memories extracted from transcript
└── Auto-sync to storage

Next Session:
├── Pre-prompt hook loads pending todos
├── Injects: "Continuing from last session: [tasks]"
└── Claude picks up where it left off
```

**Key**: No `am task` commands needed. TodoWrite IS the interface.

### 2. Plan Mode → Automatic Task Creation

```
User: "Plan authentication system"
         │
         ▼
Claude enters plan mode
├── Explores codebase
├── Creates plan (plan.md or output)
└── Exits plan mode
         │
         ▼
Post-plan hook (automatic)
├── Detects plan was created
├── Extracts tasks from plan via LLM
├── Creates tasks in AgentMem
├── Links tasks to plan
└── Injects: "Plan created with 5 tasks. Starting task 1..."
         │
         ▼
Claude works through tasks
├── Uses TodoWrite for sub-steps
├── AgentMem tracks overall progress
└── Auto-progression to next task
```

### 3. Sub-agent Coordination

**Problem**: Sub-agents flood terminal, don't share memory, crash system.

**Solution**: Task queue with coordination.

```
Instead of: 10 parallel sub-agents (chaos)
Do this:    Task queue with controlled execution

Main Agent
    │
    ├── Creates tasks in AgentMem queue
    │   Task 1: "Implement login"
    │   Task 2: "Implement logout"
    │   Task 3: "Add JWT validation"
    │
    ▼
AgentMem Task Queue
    │
    ├── Sub-agent 1 pulls Task 1
    │   ├── Reads shared context (am context)
    │   ├── Does work
    │   ├── Writes learnings (am mem add)
    │   └── Marks complete
    │
    ├── Sub-agent 2 pulls Task 2 (after 1 done)
    │   └── ...
    │
    └── Coordination, not chaos
```

---

## Storage Architecture

### Per-Project Isolation

```
project/
└── .agentmem/
    ├── agentmem.db          # SQLite - all data
    ├── agentmem.jsonl       # Git-friendly export (opt-in)
    ├── config.yaml          # Project config
    └── hooks/               # Hook scripts
```

### Qdrant Isolation (Vector DB)

Single Qdrant container serves all projects, but collections are namespaced:

```
Project A: collection = agentmem_a1b2c3d4 (hash of path)
Project B: collection = agentmem_e5f6g7h8 (different hash)
```

### Workspace Support (Multi-Repo)

```
~/workspace/my-app/              # Just a folder, no .git
│
├── .agentmem/                   # SHARED memories
│   └── "API uses JWT", "Use pnpm"
│
├── backend/                     # Separate Git repo
│   ├── .git/
│   └── .agentmem/               # LOCAL memories
│
├── mobile-ios/                  # Separate Git repo
│   ├── .git/
│   └── .agentmem/               # LOCAL memories
│
└── dashboard/                   # Separate Git repo
    ├── .git/
    └── .agentmem/               # LOCAL memories
```

**Context retrieval merges**:
1. Current folder's `.agentmem/` (local)
2. Parent folder's `.agentmem/` (shared)
3. Walk up until no more `.agentmem/` found

**Adding memories**:
- Default: saves to current project's `.agentmem/`
- `--global`: saves to parent workspace's `.agentmem/`

---

## Database Schema (2.0)

### New Tables

```sql
-- Plans from plan mode
CREATE TABLE plans (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT,                    -- Full plan content
    file_path TEXT,                  -- If saved to file
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME
);

-- Link plans to tasks
CREATE TABLE plan_tasks (
    plan_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    task_order INTEGER,              -- Order in plan
    PRIMARY KEY (plan_id, task_id),
    FOREIGN KEY (plan_id) REFERENCES plans(id),
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

-- TodoWrite state snapshots
CREATE TABLE todowrite_snapshots (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    snapshot_json TEXT NOT NULL,     -- Full TodoWrite state
    captured_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Session tracking
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    ended_at DATETIME,
    status TEXT DEFAULT 'active',    -- active, completed, compacted
    last_task_id TEXT,               -- Where we left off
    summary TEXT                     -- What was accomplished
);

-- Task status history
CREATE TABLE task_history (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    old_status TEXT,
    new_status TEXT NOT NULL,
    changed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    changed_by TEXT,                 -- 'user', 'agent', 'hook'
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);
```

### Updated Tasks Table

```sql
-- Add columns to existing tasks table
ALTER TABLE tasks ADD COLUMN plan_id TEXT REFERENCES plans(id);
ALTER TABLE tasks ADD COLUMN todowrite_id TEXT;  -- Link to TodoWrite item
ALTER TABLE tasks ADD COLUMN parent_task_id TEXT; -- For subtasks
```

---

## Hook System (2.0)

### Pre-Prompt Hook (Enhanced)

```javascript
// Runs before each user prompt

1. Load session state
   - Last session summary
   - Pending tasks from TodoWrite snapshot
   - Current plan if any

2. Load relevant memories
   - Walk up directory tree for .agentmem/ folders
   - Merge local + shared memories
   - Semantic search if query provided

3. Load protected files
   - From all .agentmem/ folders in hierarchy

4. Format injection
   ## Continuing from previous session
   - You were working on: [task]
   - Pending tasks: [list]

   ## Relevant Context
   - [memories]

   ## Protected Files
   - [files]

5. Return contextPrefix
```

### Post-Session Hook (Enhanced)

```javascript
// Runs when session ends

1. Capture final state
   - Parse transcript for last TodoWrite call
   - Save snapshot to todowrite_snapshots

2. Extract learnings
   - Send transcript to GPT-4o
   - Extract memories (decisions, corrections, gotchas)
   - Deduplicate against existing
   - Save new memories

3. Detect plans
   - Look for plan file creation in transcript
   - If found, parse into tasks
   - Save plan and tasks

4. Update session record
   - Mark session completed
   - Record what was accomplished

5. Auto-sync
   - Export to JSONL if git-sync enabled
   - Git commit if configured
```

### New: Post-Plan Hook

```javascript
// Runs when plan mode exits

1. Detect plan output
   - Check for plan file created
   - Or parse plan from Claude's output

2. Extract tasks from plan
   - Use LLM to parse plan into discrete tasks
   - Identify dependencies

3. Create tasks in AgentMem
   - Link to plan
   - Set initial priorities

4. Inject next action
   - "Plan created with N tasks"
   - "Starting task 1: [description]"
```

---

## Init Command (2.0)

```bash
$ am init

1. Check/Install Docker
   - If not installed, provide install instructions
   - Or offer to install via script

2. Check/Start Qdrant
   - Pull image if needed
   - Start container if not running
   - Create project-specific collection

3. Configure API keys
   - Check ~/.agentmem/credentials
   - Prompt for OpenAI key if missing
   - Validate key works

4. Create .agentmem/ directory
   - agentmem.db (with 2.0 schema)
   - config.yaml (with collection name)
   - .gitignore

5. Detect and install hooks
   - Auto-detect Claude Code, Cursor, etc.
   - Install appropriate hooks
   - Update agent settings files

6. Check for parent .agentmem/
   - If workspace setup detected
   - Offer to link for shared memories

Output:
✅ AgentMem initialized
   - SQLite: .agentmem/agentmem.db
   - Vectors: Qdrant collection 'agentmem_xxxxx'
   - Hooks: Claude Code (pre-prompt, post-session)
   - Shared: Linked to ../agentmem/ (if applicable)
```

---

## User Workflow (2.0)

### First Time Setup

```bash
# One time per machine
# (Docker and Qdrant auto-handled by init)

# Per project
cd my-project
am init
# That's it. Hooks installed, ready to go.
```

### Daily Work

```bash
# Just use Claude Code normally
# AgentMem works invisibly

# Optional shortcuts (but not required)
/remember decision "Use PostgreSQL"
/protect src/auth/core.ts
/sync
```

### What Happens Automatically

1. **Session Start**: Context injected (memories, pending tasks, protected files)
2. **During Session**: TodoWrite state tracked
3. **Plan Created**: Tasks auto-extracted and created
4. **Session End**: State saved, memories extracted, synced
5. **Next Session**: Picks up exactly where you left off

---

## Success Criteria

| Metric | Target |
|--------|--------|
| Manual commands needed | Zero (fully automatic) |
| TodoWrite state preserved | 100% across sessions |
| Plan → Tasks conversion | Automatic |
| Context injection latency | < 500ms |
| Sub-agent coordination | No crashes, shared memory |

---

## Implementation Phases

### Phase 1: TodoWrite Persistence
- Capture TodoWrite state in post-session hook
- Restore in pre-prompt hook
- Basic session continuity

### Phase 2: Plan Integration
- Detect plan creation
- Extract tasks from plans
- Link plans to tasks

### Phase 3: Enhanced Hooks
- Post-plan hook
- Real-time TodoWrite capture
- Auto-progression between tasks

### Phase 4: Workspace Support
- Hierarchical .agentmem/ discovery
- --global flag for shared memories
- Context merging from multiple sources

### Phase 5: Sub-agent Coordination
- Task queue system
- Shared context for sub-agents
- Controlled execution (not parallel chaos)

### Phase 6: Init Improvements
- Auto Docker/Qdrant setup
- Project collection isolation
- One-command setup

---

## Open Questions

1. **TodoWrite state format**: Need to verify exact JSON structure Claude uses
2. **Plan detection**: How reliably can we detect plan mode exit?
3. **Hook events**: Does Claude Code support post-response hooks?
4. **Sub-agent spawning**: Can we intercept/control Task tool behavior?

---

---

## Implementation as Claude Code Plugin

AgentMem 2.0 should be implemented as a **Claude Code Plugin**, not just a CLI with hooks.

### Plugin Structure

```
agentmem-plugin/
├── .claude-plugin/
│   └── plugin.json           # Plugin manifest
├── commands/                  # Slash commands
│   ├── remember.md           # /agentmem:remember
│   ├── protect.md            # /agentmem:protect
│   ├── sync.md               # /agentmem:sync
│   ├── context.md            # /agentmem:context
│   └── status.md             # /agentmem:status
├── skills/                    # Auto-invoked by Claude
│   ├── memory-persistence/
│   │   └── SKILL.md          # Persist TodoWrite, extract learnings
│   └── plan-to-tasks/
│       └── SKILL.md          # Convert plans to tasks
├── hooks/
│   └── hooks.json            # Pre-prompt, post-session hooks
├── .mcp.json                 # MCP server for am CLI
└── README.md
```

### Plugin Manifest

```json
{
  "name": "agentmem",
  "description": "Persistent memory for Claude Code - survives sessions and compaction",
  "version": "2.0.0",
  "author": {
    "name": "AgentMem"
  }
}
```

### Slash Commands

| Command | Description |
|---------|-------------|
| `/agentmem:remember <type> <title>` | Add a memory |
| `/agentmem:protect <file>` | Mark file as protected |
| `/agentmem:sync` | Sync to git |
| `/agentmem:context` | Show current context |
| `/agentmem:status` | Show AgentMem status |

### Skills (Auto-Invoked)

**memory-persistence/SKILL.md**:
```markdown
---
name: memory-persistence
description: Automatically persist session state and extract learnings.
  Use at session end or when important decisions/corrections are made.
---

When the session is ending or context is being compacted:
1. Capture current TodoWrite state
2. Extract any decisions, corrections, or gotchas from conversation
3. Run `am extract` to save learnings
4. Run `am sync` to persist

When you notice a correction, decision, or gotcha:
1. Run `am mem add <type> <title> --content "<details>"`
```

**plan-to-tasks/SKILL.md**:
```markdown
---
name: plan-to-tasks
description: Convert plans into trackable tasks.
  Use after creating an implementation plan.
---

After creating a plan:
1. Extract discrete tasks from the plan
2. For each task, run `am task create "<title>" --description "<details>"`
3. Report: "Created N tasks from plan"
4. Start on first task
```

### Hooks (hooks/hooks.json)

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "/path/to/am context --query \"$PROMPT\" --format inject"
          }
        ]
      }
    ],
    "Stop": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "/path/to/am session end --extract --sync"
          }
        ]
      }
    ]
  }
}
```

### Benefits of Plugin Approach

1. **Native Integration**: Skills auto-invoke based on context
2. **Proper Hooks**: Uses Claude Code's official hook system
3. **Distributable**: Can share via plugin marketplace
4. **Slash Commands**: Feel native to Claude Code
5. **Versioned**: Proper plugin versioning

### MCP Server Integration

The `am` CLI can be exposed as an MCP server for richer integration:

**.mcp.json**:
```json
{
  "servers": {
    "agentmem": {
      "command": "am",
      "args": ["mcp-server"],
      "description": "AgentMem memory system"
    }
  }
}
```

This allows Claude to directly call AgentMem functions without shell commands.

---

*This document captures the design decisions from brainstorming session on January 2025.*
