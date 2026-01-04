# AgentMem 2.0 Implementation Tasks

> Generated from plan: `/Users/eagleisbatman/.claude/plans/hazy-discovering-micali.md`
> Design doc: `docs/AGENTMEM-2.0-DESIGN.md`

---

## Phase 1: Plugin Structure (FIRST PRIORITY)

### 1.1 Create Plugin Manifest
- [ ] Create `plugin/.claude-plugin/plugin.json`
- [ ] Set name: "agentmem", version: "2.0.0"
- [ ] Add description and author

### 1.2 Create Slash Commands
- [ ] `plugin/commands/remember.md` - `/agentmem:remember <type> <title>`
  - Call `am mem add <type> <title> --content "$ARGUMENTS"`
- [ ] `plugin/commands/protect.md` - `/agentmem:protect <file>`
  - Call `am protect <file>`
- [ ] `plugin/commands/sync.md` - `/agentmem:sync`
  - Call `am sync`
- [ ] `plugin/commands/context.md` - `/agentmem:context`
  - Call `am context --format markdown`
- [ ] `plugin/commands/status.md` - `/agentmem:status`
  - Call `am doctor --json`

### 1.3 Create Skills
- [ ] `plugin/skills/memory-persistence/SKILL.md`
  - Auto-detect corrections, decisions, gotchas in conversation
  - Instruct to run `am mem add` when learnings detected
  - Instruct to run `am sync` at session end
  - Capture TodoWrite state context

- [ ] `plugin/skills/plan-to-tasks/SKILL.md`
  - Detect when plan mode exits or plan file created
  - Parse plan into discrete tasks
  - Run `am task create` for each task
  - Start working on first task

### 1.4 Create Hooks Configuration
- [ ] `plugin/hooks/hooks.json`
  - `UserPromptSubmit`: Inject context via `am context --query`
  - `Stop`: Run `am session end --extract --sync`

### 1.5 Create MCP Configuration
- [ ] `plugin/.mcp.json`
  - Configure `am mcp-server` command

### 1.6 Test Plugin
- [ ] Test with `claude --plugin-dir ./plugin`
- [ ] Verify slash commands work
- [ ] Verify skills are recognized
- [ ] Verify hooks fire correctly

---

## Phase 2: Database Schema Updates

### 2.1 Add Plans Table
- [ ] `src/db/migrations.rs`: Add `plans` table
  - id (TEXT PRIMARY KEY)
  - title (TEXT NOT NULL)
  - content (TEXT)
  - file_path (TEXT)
  - created_at (DATETIME)
  - updated_at (DATETIME)

### 2.2 Add Plan-Tasks Link Table
- [ ] Add `plan_tasks` table
  - plan_id (TEXT, FK to plans)
  - task_id (TEXT, FK to tasks)
  - task_order (INTEGER)
  - PRIMARY KEY (plan_id, task_id)

### 2.3 Add TodoWrite Snapshots Table
- [ ] Add `todowrite_snapshots` table
  - id (TEXT PRIMARY KEY)
  - session_id (TEXT NOT NULL)
  - snapshot_json (TEXT NOT NULL)
  - captured_at (DATETIME)

### 2.4 Add Sessions Table
- [ ] Add `sessions` table
  - id (TEXT PRIMARY KEY)
  - started_at (DATETIME)
  - ended_at (DATETIME)
  - status (TEXT) - active, completed, compacted
  - last_task_id (TEXT)
  - summary (TEXT)

### 2.5 Add Task History Table
- [ ] Add `task_history` table
  - id (TEXT PRIMARY KEY)
  - task_id (TEXT, FK to tasks)
  - old_status (TEXT)
  - new_status (TEXT NOT NULL)
  - changed_at (DATETIME)
  - changed_by (TEXT) - user, agent, hook

### 2.6 Update Tasks Table
- [ ] Add `plan_id` column (FK to plans)
- [ ] Add `parent_task_id` column (for subtasks)

### 2.7 Run Migration
- [ ] Test migration on fresh database
- [ ] Test migration on existing database

---

## Phase 3: Skill Implementation

### 3.1 Memory Persistence Skill Content
- [ ] Write detailed SKILL.md instructions:
  - When to trigger memory extraction
  - How to identify corrections, decisions, gotchas
  - Format for `am mem add` commands
  - When to call `am sync`

### 3.2 Plan-to-Tasks Skill Content
- [ ] Write detailed SKILL.md instructions:
  - How to detect plan completion
  - How to parse plan into tasks
  - Format for `am task create` commands
  - How to link tasks to plan

### 3.3 Test Skills
- [ ] Test memory persistence triggers correctly
- [ ] Test plan-to-tasks extracts tasks accurately

---

## Phase 4: CLI Enhancements

### 4.1 Add MCP Server Mode
- [ ] Create `src/mcp/mod.rs`
- [ ] Create `src/mcp/server.rs`
- [ ] Implement MCP protocol for `am` commands
- [ ] Add `mcp-server` subcommand to `main.rs`

### 4.2 Add Workspace Discovery
- [ ] `src/config/`: Add `find_agentmem_dirs()` function
- [ ] Walk up directory tree looking for `.agentmem/`
- [ ] Return list of paths (local first, then parents)

### 4.3 Add --global Flag
- [ ] `src/main.rs`: Add `--global` to `mem add` command
- [ ] When `--global`, save to parent `.agentmem/` if exists
- [ ] Error if no parent `.agentmem/` found

### 4.4 Hierarchical Context Merging
- [ ] `src/retrieval/context.rs`: Merge context from all `.agentmem/` dirs
- [ ] Local memories first, then parent memories
- [ ] Deduplicate by ID

### 4.5 Project-Specific Qdrant Collections
- [ ] Generate collection name from project path hash
- [ ] Store in `config.yaml` on init
- [ ] Use in all Qdrant operations

### 4.6 Add Plan Commands
- [ ] `am plan create <title>` - Create plan record
- [ ] `am plan list` - List plans
- [ ] `am plan show <id>` - Show plan with linked tasks

### 4.7 Add Session Commands (enhance existing)
- [ ] `am session start` - Create session record
- [ ] `am session end --extract --sync` - End with extraction
- [ ] `am session snapshot <json>` - Save TodoWrite state

---

## Phase 5: Init Command Overhaul

### 5.1 Plugin Installation
- [ ] `src/init.rs`: Add plugin installation logic
- [ ] Copy plugin to `~/.claude/plugins/agentmem/`
- [ ] Or symlink for development

### 5.2 Docker Auto-Detection
- [ ] Check if Docker is installed
- [ ] If not, print installation instructions
- [ ] If installed but not running, prompt to start

### 5.3 Qdrant Auto-Setup
- [ ] Check if Qdrant container exists
- [ ] If not, pull image and create container
- [ ] If exists but stopped, start it
- [ ] Wait for health check

### 5.4 Project Collection Setup
- [ ] Generate unique collection name
- [ ] Create collection in Qdrant
- [ ] Store in config.yaml

### 5.5 Workspace Detection
- [ ] Check for parent `.agentmem/`
- [ ] If found, offer to link for shared memories
- [ ] Update config to reference parent

### 5.6 Remove Legacy Hook Installation
- [ ] Remove code that creates `.agentmem/hooks/`
- [ ] Remove code that updates `.claude/settings.json`

---

## Phase 6: Remove Legacy Hooks

### 6.1 Clean Up templates.rs
- [ ] Remove `CLAUDE_PRE_PROMPT_HOOK` template
- [ ] Remove `CLAUDE_POST_SESSION_HOOK` template
- [ ] Remove other agent hook templates
- [ ] Keep only plugin-related code if any

### 6.2 Simplify service.rs
- [ ] Remove `install_claude_code_hooks()` function
- [ ] Remove `install_*_hooks()` functions
- [ ] Keep `list_hooks()` for backward compat (check plugin)
- [ ] Update `detect_installed_agents()` if needed

### 6.3 Update Documentation
- [ ] Update README.md for plugin installation
- [ ] Update CLAUDE.md
- [ ] Archive old hook documentation

---

## Testing & Validation

### T1. Plugin Testing
- [ ] Test fresh install with plugin
- [ ] Test slash commands work
- [ ] Test skills trigger correctly
- [ ] Test hooks fire and return correct data

### T2. Migration Testing
- [ ] Test fresh database creation
- [ ] Test upgrade from 1.x database
- [ ] Verify all tables created correctly

### T3. Workspace Testing
- [ ] Test single project setup
- [ ] Test workspace with multiple projects
- [ ] Test `--global` flag
- [ ] Test context merging

### T4. Integration Testing
- [ ] Full workflow: init → add memories → sync
- [ ] Plan mode → tasks creation
- [ ] Session persistence across restarts

---

## Documentation

### D1. Update README.md
- [ ] New installation instructions (plugin-based)
- [ ] Remove hook installation section
- [ ] Add plugin commands reference

### D2. Create MIGRATION.md
- [ ] Guide for 1.x users upgrading to 2.0
- [ ] Breaking changes list
- [ ] Data migration steps (if any)

### D3. Update CLAUDE.md
- [ ] Reflect new architecture
- [ ] Update development commands

---

## Priority Order

1. **Phase 1.1-1.4**: Plugin structure (foundation)
2. **Phase 1.6**: Test plugin works
3. **Phase 2**: Database schema (needed for features)
4. **Phase 3**: Skill content (core automation)
5. **Phase 4.1**: MCP server (optional, can defer)
6. **Phase 4.2-4.4**: Workspace support
7. **Phase 5**: Init overhaul
8. **Phase 6**: Legacy cleanup
9. **Testing & Documentation**

---

## Notes

- Clean break from 1.x: No backward compatibility
- Plugin-first approach: Everything through Claude Code plugin
- MCP server is optional enhancement (can use CLI calls initially)
