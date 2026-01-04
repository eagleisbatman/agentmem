# AgentMem 2.0 Implementation Tasks

> Generated from plan: `/Users/eagleisbatman/.claude/plans/hazy-discovering-micali.md`
> Design doc: `docs/AGENTMEM-2.0-DESIGN.md`

---

## Phase 1: Plugin Structure (FIRST PRIORITY) ✅

### 1.1 Create Plugin Manifest
- [x] Create `plugin/.claude-plugin/plugin.json`
- [x] Set name: "agentmem", version: "2.0.0"
- [x] Add description and author

### 1.2 Create Slash Commands
- [x] `plugin/commands/remember.md` - `/agentmem:remember <type> <title>`
- [x] `plugin/commands/protect.md` - `/agentmem:protect <file>`
- [x] `plugin/commands/sync.md` - `/agentmem:sync`
- [x] `plugin/commands/context.md` - `/agentmem:context`
- [x] `plugin/commands/status.md` - `/agentmem:status`

### 1.3 Create Skills
- [x] `plugin/skills/memory-persistence/SKILL.md`
- [x] `plugin/skills/plan-to-tasks/SKILL.md`

### 1.4 Create Hooks Configuration
- [x] `plugin/hooks/hooks.json`

### 1.5 Create MCP Configuration
- [x] `plugin/.mcp.json`

### 1.6 Test Plugin
- [ ] Test with `claude --plugin-dir ./plugin`
- [ ] Verify slash commands work
- [ ] Verify skills are recognized
- [ ] Verify hooks fire correctly

---

## Phase 2: Database Schema Updates ✅

### 2.1 Add Plans Table
- [x] `src/db/migrations.rs`: Add `plans` table

### 2.2 Add Plan-Tasks Link Table
- [x] Add `plan_tasks` table

### 2.3 Add TodoWrite Snapshots Table
- [x] Add `todowrite_snapshots` table

### 2.4 Add Sessions Table
- [x] Add `sessions` table

### 2.5 Add Task History Table
- [x] Add `task_history` table

### 2.6 Update Tasks Table
- [ ] Add `plan_id` column (FK to plans) - *Deferred: can use plan_tasks junction*
- [ ] Add `parent_task_id` column (for subtasks) - *Deferred: less critical*

### 2.7 Run Migration
- [x] Test migration on fresh database
- [x] Test migration on existing database

---

## Phase 3: Skill Implementation ✅

### 3.1 Memory Persistence Skill Content
- [x] Write detailed SKILL.md instructions

### 3.2 Plan-to-Tasks Skill Content
- [x] Write detailed SKILL.md instructions

### 3.3 Test Skills
- [ ] Test memory persistence triggers correctly
- [ ] Test plan-to-tasks extracts tasks accurately

---

## Phase 4: CLI Enhancements ✅

### 4.1 Add MCP Server Mode
- [x] Create `src/mcp/mod.rs`
- [x] Create `src/mcp/server.rs`
- [x] Implement MCP protocol for `am` commands
- [x] Add `mcp-server` subcommand to `main.rs`

### 4.2 Add Workspace Discovery
- [x] `src/config/`: Add `find_agentmem_dir()` function
- [x] Walk up directory tree looking for `.agentmem/`
- [x] Return path to nearest `.agentmem/`

### 4.3 Add --global Flag
- [ ] `src/main.rs`: Add `--global` to `mem add` command - *Deferred: less critical*

### 4.4 Hierarchical Context Merging
- [x] `src/retrieval/context.rs`: Use hierarchical discovery
- [ ] Merge context from all `.agentmem/` dirs - *Deferred: single nearest for now*

### 4.5 Project-Specific Qdrant Collections
- [x] Generate collection name from project path hash
- [x] Store in `config.yaml` on init
- [x] Use in all Qdrant operations

### 4.6 Add Plan Commands
- [x] `am plan create <title>` - Create plan record
- [x] `am plan list` - List plans
- [x] `am plan show <id>` - Show plan with linked tasks
- [x] `am plan complete <id>` - Mark plan complete
- [x] `am plan abandon <id>` - Abandon plan
- [x] `am plan link <plan_id> <task_id>` - Link task to plan
- [x] `am plan active` - Show active plan

### 4.7 Add Session Commands (enhance existing)
- [x] `am session start` - Create session record
- [x] `am session end` - End session
- [x] `am session list` - List sessions
- [x] `am session save-todos <json>` - Save TodoWrite state
- [x] `am session get-todos` - Get most recent TodoWrite snapshot

### 4.8 Add Task Commands (new)
- [x] `am task update <id> <status>` - Update task status with history
- [x] `am task history <id>` - Show task history
- [x] `am task show <id>` - Show task details

---

## Phase 5: Init Command Overhaul ✅

### 5.1 Plugin Installation
- [x] `src/init.rs`: Add plugin installation logic
- [x] Copy plugin to `~/.claude/plugins/agentmem/`

### 5.2 Docker Auto-Detection
- [x] Check if Docker is installed
- [x] If not, print installation instructions
- [x] If installed but not running, prompt to start

### 5.3 Qdrant Auto-Setup
- [x] Check if Qdrant container exists
- [x] If not, pull image and create container
- [x] If exists but stopped, start it
- [x] Wait for health check

### 5.4 Project Collection Setup
- [x] Generate unique collection name
- [x] Store in config.yaml

### 5.5 Workspace Detection
- [x] Check for parent `.agentmem/` via hierarchical discovery
- [ ] Offer to link for shared memories - *Deferred*

### 5.6 Remove Legacy Hook Installation
- [x] Added deprecation notice for legacy hooks
- [ ] Full removal - *Deferred for backward compatibility*

---

## Phase 6: Remove Legacy Hooks ⚠️ PARTIAL

### 6.1 Clean Up templates.rs
- [ ] Remove hook templates - *Deferred: kept for backward compat*

### 6.2 Simplify service.rs
- [x] Added deprecation notice to `install_claude_code_hooks()`
- [ ] Remove functions entirely - *Deferred*

### 6.3 Update Documentation
- [ ] Update README.md for plugin installation
- [ ] Update CLAUDE.md
- [ ] Archive old hook documentation

---

## Phase 7: Sync & Real-time Hooks ✅ NEW

### 7.1 Real-time Hooks
- [x] PostToolUse hook for Write|TodoWrite tools
- [x] Capture TodoWrite state on every usage
- [x] Detect plan file creation → trigger task extraction
- [x] Stop hook reads transcript_path from Claude Code

### 7.2 Plan → Tasks Automation
- [x] `am plan extract-tasks --file <plan.md>` command
- [x] GPT-4o parses plan into discrete tasks
- [x] Creates tasks with priority and order
- [x] Links tasks to active plan automatically

### 7.3 Git Sync Improvements
- [x] Pull before push (prevents conflicts)
- [x] Auto-import after pull if JSONL changed
- [x] Conflict resolution guidance

### 7.4 Cross-Machine Sync
- [x] `am import --embed` regenerates embeddings
- [x] Auto-import on init (detects JSONL from git clone)
- [x] Full sync workflow: export → pull → import → merge → push

---

## Testing & Validation

### T1. Plugin Testing
- [ ] Test fresh install with plugin
- [ ] Test slash commands work
- [ ] Test skills trigger correctly
- [x] Test hooks fire and return correct data (PostToolUse, Stop)

### T2. Migration Testing
- [x] Test fresh database creation
- [x] Test upgrade from existing database
- [x] Verify all tables created correctly

### T3. Workspace Testing
- [x] Test single project setup
- [x] Test hierarchical discovery from subdirectory
- [ ] Test `--global` flag - *Deferred*
- [ ] Test context merging from multiple dirs - *Deferred*

### T4. Integration Testing
- [x] Full workflow: init → add memories → sync
- [x] Plan mode → tasks creation (via `am plan extract-tasks`)
- [x] Session persistence across restarts (via hooks)
- [x] Sub-agent memory sharing (tested - agents can read/write memories)

---

## Documentation ✅

### D1. Update README.md
- [x] New installation instructions (plugin-based)
- [x] Remove hook installation section
- [x] Add plugin commands reference

### D2. Create MIGRATION.md
- [x] Guide for 1.x users upgrading to 2.0
- [x] Breaking changes list
- [x] Data migration steps (if any)

### D3. Update CLAUDE.md
- [x] Reflect new architecture
- [x] Update development commands

---

## Summary

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 1: Plugin Structure | ✅ Complete | Plugin created, needs testing |
| Phase 2: Database Schema | ✅ Complete | All 5 tables added |
| Phase 3: Skill Implementation | ✅ Complete | Skills written, needs testing |
| Phase 4: CLI Enhancements | ✅ Complete | MCP server, workspace support, plan/task commands |
| Phase 5: Init Overhaul | ✅ Complete | Docker/Qdrant auto-setup, plugin install |
| Phase 6: Legacy Hooks | ⚠️ Partial | Deprecation notice added, full removal deferred |
| Phase 7: Sync & Hooks | ✅ Complete | Real-time hooks, plan extraction, git sync |
| Testing | ✅ Mostly Complete | Integration tests pass, plugin testing pending |
| Documentation | ✅ Complete | README, CLAUDE.md, MIGRATION.md updated |

---

## Deferred Items

These items were intentionally deferred as less critical:

1. `--global` flag for workspace-wide memories
2. Multi-directory context merging
3. Full legacy hook removal (kept for backward compat)
4. `plan_id` and `parent_task_id` columns in tasks table
5. Documentation updates
