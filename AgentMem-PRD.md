# Product Requirements Document: AgentMem

## Agent Memory System for Persistent Context in AI Coding Agents

**Version**: 1.0.0-draft  
**Author**: Claude (with Gautam)  
**Date**: December 24, 2024  
**Status**: Draft for Review

---

## Executive Summary

AgentMem is a CLI-based memory system that gives AI coding agents (Claude Code, Cursor, Windsurf, etc.) persistent context across sessions and compactions. It combines task tracking (inspired by Beads) with semantic memory retrieval (inspired by Memory Lane) to solve the "agent amnesia" problem.

**The Problem**: AI coding agents forget everything between sessions and after context compaction — infrastructure details, architectural decisions, existing scripts, protected files, and project context. This leads to:
- Destructive changes to working code
- Recreation of existing utilities
- Repeated mistakes
- Lost institutional knowledge

**The Solution**: A unified CLI tool (`am`) that:
1. Tracks tasks with dependencies (like Beads)
2. Stores learnings with semantic embeddings (like Memory Lane)
3. Injects relevant context on each prompt via hooks
4. Extracts learnings automatically from session transcripts
5. Syncs via git for cross-machine collaboration

---

## Table of Contents

1. [Goals & Non-Goals](#1-goals--non-goals)
2. [User Personas](#2-user-personas)
3. [Core Concepts](#3-core-concepts)
4. [System Architecture](#4-system-architecture)
5. [Data Models](#5-data-models)
6. [CLI Specification](#6-cli-specification)
7. [Hook System](#7-hook-system)
8. [Memory Extraction](#8-memory-extraction)
9. [Retrieval System](#9-retrieval-system)
10. [Storage & Sync](#10-storage--sync)
11. [Integration with Agents](#11-integration-with-agents)
12. [Technical Requirements](#12-technical-requirements)
13. [Implementation Phases](#13-implementation-phases)
14. [Success Metrics](#14-success-metrics)
15. [Open Questions](#15-open-questions)

---

## 1. Goals & Non-Goals

### Goals

1. **Survive Compaction**: Context persists across session compactions and restarts
2. **Semantic Retrieval**: Inject only relevant memories per query (not everything)
3. **Automatic Extraction**: Learn from sessions without manual documentation
4. **Task Tracking**: Know what needs to be done, with dependencies
5. **Protection System**: Prevent modification of sensitive files
6. **Tool Registry**: Remember existing scripts/utilities
7. **Git-Based Sync**: Share memory across machines via git
8. **Low Token Overhead**: Inject minimal context per query (<2000 tokens)
9. **Agent Agnostic**: Work with Claude Code, Cursor, Windsurf, Aider, etc.
10. **Zero External Services**: No PostgreSQL, no cloud APIs (optional embeddings API)

### Non-Goals

1. Not a replacement for version control
2. Not a full project management system
3. Not a knowledge base for general information
4. Not a chat history storage system
5. Not a code indexing/search tool (use existing tools for that)

---

## 2. User Personas

### Primary: Developer Using AI Coding Agent

- Uses Claude Code or similar daily
- Works on complex projects with multiple services
- Frustrated by agent forgetting context
- Wants agent to "just know" their project

### Secondary: AI Coding Agent

- Needs to orient quickly after compaction
- Must know what's protected, what exists, what was decided
- Should file learnings automatically
- Queries for relevant context per task

---

## 3. Core Concepts

### 3.1 Tasks (from Beads)

Tasks are units of work with status, priority, and dependencies.

```
Task {
  id: "am-a1b2"
  title: "Fix authentication bug"
  status: open | in_progress | closed
  priority: 0-4 (0 = highest)
  type: bug | feature | task | epic | chore
  dependencies: [task_ids]
  labels: [strings]
  created_at, updated_at, closed_at
}
```

**Key Features**:
- `am ready` → Shows unblocked tasks
- Dependency types: blocks, related, parent-child, discovered-from
- Hierarchical IDs: `am-a1b2.1`, `am-a1b2.2` for subtasks

### 3.2 Memories (from Memory Lane)

Memories are learnings extracted from sessions.

```
Memory {
  id: uuid
  type: correction | decision | infrastructure | tool | protected | pattern | insight | gotcha
  title: "Railway API endpoint"
  content: "Production API is at https://api.farmerchat.railway.app"
  
  # For semantic search
  embedding: vector(768 or 1024)
  source_chunk: "User said: the Railway API is at..."
  
  # Metadata
  confidence: 0-100
  times_recalled: int
  last_recalled_at: datetime
  entities: [{type, name, slug}]
  
  created_at, updated_at
}
```

**Memory Types** (prioritized):

| Priority | Type | Description | Example |
|----------|------|-------------|---------|
| Critical | `protected` | Files not to modify | "Never modify src/prompts/system.md" |
| Critical | `correction` | User corrected agent | "Use friendly tone with Amy" |
| High | `infrastructure` | URLs, endpoints, configs | "Railway DB in .env as DATABASE_URL" |
| High | `tool` | Existing scripts/utilities | "scripts/translate.ts for translations" |
| High | `decision` | Architectural choices + why | "Prisma over raw SQL for type safety" |
| Medium | `pattern` | Repeated behaviors | "Run tests before pushing" |
| Medium | `gotcha` | Things that broke | "Raw SQL breaks Prisma migrations" |
| Lower | `insight` | Non-obvious discoveries | "Viettel AI better for Vietnamese" |

### 3.3 Entities

Named things that appear in memories and can be matched.

```
Entity {
  type: person | service | project | file | business
  name: "Railway"
  slug: "infrastructure:railway"
  aliases: ["railway.app", "Railway.app"]
}
```

### 3.4 Context Injection

On each prompt, relevant context is injected:

```
┌─────────────────────────────────────────────────────┐
│ INJECTED CONTEXT (< 2000 tokens)                    │
├─────────────────────────────────────────────────────┤
│ ## Current Tasks                                    │
│ - [P1] am-a1b2: Fix auth bug (in_progress)         │
│ - [P2] am-c3d4: Add OAuth (blocked by am-a1b2)     │
│                                                     │
│ ## Relevant Memories                                │
│ - [infrastructure] Railway API: https://...         │
│ - [protected] Don't modify: src/prompts/system.md   │
│ - [tool] Use scripts/translate.ts for translations  │
│                                                     │
│ ## Protected Files                                  │
│ - src/prompts/*.md                                  │
│ - prisma/schema.prisma                              │
└─────────────────────────────────────────────────────┘
```

---

## 4. System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              AgentMem System                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐                   │
│  │   CLI (am)  │────▶│   Core DB   │◀────│   Hooks     │                   │
│  │             │     │  (SQLite)   │     │  (JS/Python)│                   │
│  └─────────────┘     └──────┬──────┘     └──────┬──────┘                   │
│                             │                   │                           │
│                             ▼                   ▼                           │
│                      ┌─────────────┐     ┌─────────────┐                   │
│                      │   JSONL     │     │  Embedding  │                   │
│                      │  (git sync) │     │  Service    │                   │
│                      └─────────────┘     └─────────────┘                   │
│                             │                   │                           │
│                             ▼                   ▼                           │
│                      ┌─────────────┐     ┌─────────────┐                   │
│                      │    Git      │     │ Local Ollama│                   │
│                      │   Remote    │     │ or Gemini   │                   │
│                      └─────────────┘     └─────────────┘                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘

Data Flow:
──────────

1. User Prompt → Hook intercepts
                      │
                      ▼
2. Hook calls: am context --query "user prompt" --json
                      │
                      ▼
3. CLI returns: { tasks: [...], memories: [...], protected: [...] }
                      │
                      ▼
4. Hook injects context into agent's prompt
                      │
                      ▼
5. Agent works with full context
                      │
                      ▼
6. Session ends → Hook calls: am extract --transcript <file>
                      │
                      ▼
7. CLI extracts memories, updates DB
                      │
                      ▼
8. am sync → Commits to git
```

---

## 5. Data Models

### 5.1 Database Schema (SQLite)

```sql
-- Tasks (Beads-style)
CREATE TABLE tasks (
  id TEXT PRIMARY KEY,              -- "am-a1b2" or "am-a1b2.1"
  title TEXT NOT NULL,
  description TEXT,
  status TEXT DEFAULT 'open',       -- open, in_progress, closed
  priority INTEGER DEFAULT 2,       -- 0-4
  type TEXT DEFAULT 'task',         -- bug, feature, task, epic, chore
  labels TEXT,                      -- JSON array
  assignee TEXT,
  notes TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  closed_at DATETIME,
  closed_reason TEXT
);

-- Task dependencies
CREATE TABLE task_dependencies (
  from_id TEXT NOT NULL,
  to_id TEXT NOT NULL,
  type TEXT DEFAULT 'blocks',       -- blocks, related, parent-child, discovered-from
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (from_id, to_id, type)
);

-- Memories
CREATE TABLE memories (
  id TEXT PRIMARY KEY,              -- UUID
  type TEXT NOT NULL,               -- correction, decision, infrastructure, etc.
  title TEXT NOT NULL,
  content TEXT,
  source_chunk TEXT,                -- Original transcript excerpt
  confidence INTEGER DEFAULT 70,    -- 0-100
  times_recalled INTEGER DEFAULT 0,
  first_observed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  last_observed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  last_recalled_at DATETIME,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Memory embeddings (separate for flexibility)
CREATE TABLE memory_embeddings (
  memory_id TEXT PRIMARY KEY,
  embedding BLOB,                   -- Serialized vector
  model TEXT,                       -- "gemini-embedding-001" or "mxbai-embed-large"
  dimensions INTEGER,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (memory_id) REFERENCES memories(id)
);

-- Memory entities (for entity-based retrieval)
CREATE TABLE memory_entities (
  memory_id TEXT NOT NULL,
  entity_type TEXT NOT NULL,        -- person, service, project, file
  entity_name TEXT NOT NULL,
  entity_slug TEXT NOT NULL,        -- normalized: "infrastructure:railway"
  FOREIGN KEY (memory_id) REFERENCES memories(id)
);

-- Known entities (for resolution)
CREATE TABLE entities (
  slug TEXT PRIMARY KEY,            -- "person:gautam", "service:railway"
  type TEXT NOT NULL,
  name TEXT NOT NULL,
  aliases TEXT,                     -- JSON array
  metadata TEXT,                    -- JSON object
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Session recalls (for feedback and analytics)
CREATE TABLE session_recalls (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  memory_id TEXT NOT NULL,
  query TEXT,
  similarity REAL,
  source TEXT,                      -- "semantic" or "entity"
  feedback INTEGER,                 -- 1 = positive, -1 = negative, 0 = none
  recalled_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (memory_id) REFERENCES memories(id)
);

-- Protected files (quick access)
CREATE TABLE protected_files (
  pattern TEXT PRIMARY KEY,         -- "src/prompts/*.md" or exact path
  reason TEXT,
  added_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Registered tools
CREATE TABLE tools (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  location TEXT NOT NULL,           -- File path or command
  description TEXT,
  usage TEXT,                       -- Usage example
  added_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Configuration
CREATE TABLE config (
  key TEXT PRIMARY KEY,
  value TEXT
);

-- Indexes
CREATE INDEX idx_memories_type ON memories(type);
CREATE INDEX idx_memories_last_recalled ON memories(last_recalled_at);
CREATE INDEX idx_memory_entities_slug ON memory_entities(entity_slug);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_priority ON tasks(priority);
```

### 5.2 JSONL Export Format (for git sync)

```jsonl
{"_type":"task","id":"am-a1b2","title":"Fix auth","status":"open","priority":1,...,"_ts":"2024-12-24T10:00:00Z"}
{"_type":"memory","id":"uuid","type":"infrastructure","title":"Railway API",...,"_ts":"2024-12-24T10:00:00Z"}
{"_type":"entity","slug":"service:railway","name":"Railway",...,"_ts":"2024-12-24T10:00:00Z"}
{"_type":"protected","pattern":"src/prompts/*.md","reason":"Working prompts","_ts":"2024-12-24T10:00:00Z"}
{"_type":"tool","id":"translate","location":"scripts/translate.ts",...,"_ts":"2024-12-24T10:00:00Z"}
```

---

## 6. CLI Specification

### 6.1 Command Overview

```
am - Agent Memory CLI

COMMANDS:
  init          Initialize AgentMem in current project
  
  # Task Management (Beads-style)
  task create   Create a new task
  task list     List tasks with filters
  task show     Show task details
  task update   Update task fields
  task close    Close a task
  task ready    Show tasks ready to work on
  task dep      Manage dependencies
  
  # Memory Management
  mem add       Add a memory manually
  mem list      List memories with filters
  mem show      Show memory details
  mem search    Semantic search memories
  mem forget    Remove a memory
  mem types     List memory types
  
  # Quick Commands (shortcuts)
  protect       Mark file as protected
  tool          Register a script/utility
  infra         Add infrastructure detail
  decide        Record a decision
  gotcha        Record a gotcha/mistake
  
  # Context & Retrieval
  context       Get context for a query (used by hooks)
  inject        Generate context injection block
  
  # Extraction
  extract       Extract memories from transcript
  
  # Sync & Export
  sync          Sync with git
  export        Export to JSONL
  import        Import from JSONL
  
  # Utilities
  doctor        Check system health
  config        Manage configuration
  stats         Show statistics
  
  # Agent Integration
  onboard       Show integration instructions
  hook          Manage hooks
```

### 6.2 Detailed Command Specifications

#### `am init`

```bash
am init [options]

Options:
  --quiet           Non-interactive mode
  --embedding       Embedding provider: ollama, gemini, openai, none (default: none)
  --model           Embedding model (default: depends on provider)

Examples:
  am init
  am init --embedding ollama --model mxbai-embed-large
  am init --embedding gemini
```

Creates:
```
.agentmem/
├── agentmem.db          # SQLite database
├── agentmem.jsonl       # Git-synced data
├── config.yaml          # Configuration
├── .gitignore           # Ignores db files
└── hooks/               # Hook scripts
    ├── pre-prompt.js
    └── post-session.js
```

#### `am task create`

```bash
am task create <title> [options]

Options:
  -d, --description   Task description
  -p, --priority      Priority 0-4 (default: 2)
  -t, --type          bug|feature|task|epic|chore (default: task)
  -l, --labels        Comma-separated labels
  -a, --assignee      Assignee
  --parent            Parent task ID (creates subtask)
  --blocked-by        Comma-separated blocking task IDs
  --json              Output JSON

Examples:
  am task create "Fix auth bug" -p 1 -t bug
  am task create "Add OAuth" --blocked-by am-a1b2
  am task create "Subtask" --parent am-a1b2
```

#### `am task ready`

```bash
am task ready [options]

Options:
  --limit             Max results (default: 10)
  --priority          Filter by priority
  --type              Filter by type
  --label             Filter by label
  --json              Output JSON

Examples:
  am task ready
  am task ready --priority 1 --json
```

#### `am mem add`

```bash
am mem add <type> <title> [options]

Types:
  correction      User corrected agent behavior
  decision        Architectural decision with reasoning
  infrastructure  URL, endpoint, config
  tool            Script or utility
  protected       File not to modify
  pattern         Repeated behavior
  gotcha          Something that broke
  insight         Discovery

Options:
  -c, --content       Full content/description
  --source            Source transcript excerpt
  --confidence        Confidence 0-100 (default: 70)
  --entity            Add entity reference (type:name)
  --json              Output JSON

Examples:
  am mem add infrastructure "Railway API" -c "https://api.farmerchat.railway.app"
  am mem add decision "Use Prisma" -c "Type safety, works with Railway" --confidence 90
  am mem add correction "Casual tone with Amy" --entity person:amy
```

#### `am protect`

```bash
am protect <path> [reason]

Shortcuts for common patterns:
  am protect src/prompts/system.md "Working prompt"
  am protect "prisma/schema.prisma" "DB schema"
  am protect "*.env*" "Environment files"
```

#### `am tool`

```bash
am tool <location> <description> [usage]

Examples:
  am tool scripts/translate.ts "Translation via Gemini" "npx ts-node scripts/translate.ts en vi"
  am tool "npm run build" "Production build"
```

#### `am context`

```bash
am context [options]

Options:
  --query             User query (for semantic search)
  --task              Current task ID
  --file              File being worked on
  --limit-memories    Max memories (default: 5)
  --limit-tasks       Max tasks (default: 3)
  --format            text|json|markdown (default: markdown)
  --json              Output JSON

Examples:
  am context --query "fix the authentication bug" --json
  am context --task am-a1b2 --file src/auth/login.ts
```

Output:
```json
{
  "tasks": [
    {"id": "am-a1b2", "title": "Fix auth bug", "status": "in_progress", "priority": 1}
  ],
  "memories": [
    {"type": "infrastructure", "title": "Railway API", "content": "https://...", "similarity": 0.85},
    {"type": "tool", "title": "Auth utils", "content": "src/utils/auth.ts", "similarity": 0.72}
  ],
  "protected": [
    {"pattern": "src/prompts/*.md", "reason": "Working prompts"}
  ],
  "tools": [
    {"name": "translate", "location": "scripts/translate.ts"}
  ]
}
```

#### `am extract`

```bash
am extract [options]

Options:
  --transcript        Path to transcript file
  --stdin             Read from stdin
  --dry-run           Show what would be extracted
  --auto-confirm      Don't ask for confirmation
  --json              Output JSON

Examples:
  am extract --transcript session.jsonl
  cat transcript.txt | am extract --stdin
  am extract --transcript session.jsonl --dry-run
```

#### `am search`

```bash
am mem search <query> [options]

Options:
  --type              Filter by memory type
  --entity            Filter by entity
  --limit             Max results (default: 10)
  --threshold         Similarity threshold 0-1 (default: 0.5)
  --json              Output JSON

Examples:
  am mem search "railway api endpoint"
  am mem search "authentication" --type decision,gotcha
  am mem search "translation" --limit 5 --json
```

#### `am sync`

```bash
am sync [options]

Options:
  --push              Also git push after commit
  --message           Custom commit message

What it does:
  1. Export DB to JSONL
  2. git add .agentmem/agentmem.jsonl
  3. git commit -m "agentmem: sync"
  4. (if --push) git push
```

---

## 7. Hook System

### 7.1 Hook Types

| Hook | Trigger | Purpose |
|------|---------|---------|
| `pre-prompt` | Before each user prompt | Inject context |
| `post-tool` | After file read/edit | Inject file-related context |
| `post-session` | Session end | Extract memories |
| `post-compact` | After compaction | Reminder to check context |

### 7.2 Pre-Prompt Hook (Claude Code)

```javascript
// .agentmem/hooks/pre-prompt.js

const { execSync } = require('child_process');

module.exports = {
  event: 'UserPromptSubmit',
  
  async handler({ prompt, files }) {
    try {
      // Get context from CLI
      const result = execSync(
        `am context --query "${prompt.replace(/"/g, '\\"')}" --json`,
        { encoding: 'utf-8', timeout: 5000 }
      );
      
      const context = JSON.parse(result);
      
      // Format as markdown
      let injection = '';
      
      if (context.protected.length > 0) {
        injection += '## ⚠️ Protected Files\n';
        injection += 'Ask before modifying:\n';
        context.protected.forEach(p => {
          injection += `- \`${p.pattern}\` — ${p.reason}\n`;
        });
        injection += '\n';
      }
      
      if (context.tasks.length > 0) {
        injection += '## Current Tasks\n';
        context.tasks.forEach(t => {
          injection += `- [P${t.priority}] ${t.id}: ${t.title} (${t.status})\n`;
        });
        injection += '\n';
      }
      
      if (context.memories.length > 0) {
        injection += '## Relevant Context\n';
        context.memories.forEach(m => {
          injection += `- [${m.type}] ${m.title}: ${m.content}\n`;
        });
        injection += '\n';
      }
      
      if (context.tools.length > 0) {
        injection += '## Available Tools\n';
        context.tools.forEach(t => {
          injection += `- \`${t.location}\` — ${t.description}\n`;
        });
      }
      
      return {
        contextPrefix: injection.trim() ? `\n---\n${injection}---\n` : ''
      };
      
    } catch (error) {
      console.error('AgentMem hook error:', error.message);
      return {};
    }
  }
};
```

### 7.3 Post-Session Hook

```javascript
// .agentmem/hooks/post-session.js

const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

module.exports = {
  event: 'SessionEnd',
  
  async handler({ transcript, sessionId }) {
    try {
      // Write transcript to temp file
      const tempFile = path.join('/tmp', `am-transcript-${sessionId}.jsonl`);
      fs.writeFileSync(tempFile, JSON.stringify(transcript));
      
      // Extract memories (async, don't block)
      spawn('am', ['extract', '--transcript', tempFile, '--auto-confirm'], {
        detached: true,
        stdio: 'ignore'
      }).unref();
      
      // Sync to git
      spawn('am', ['sync'], {
        detached: true,
        stdio: 'ignore'
      }).unref();
      
    } catch (error) {
      console.error('AgentMem post-session error:', error.message);
    }
  }
};
```

### 7.4 Hook Configuration

```yaml
# .agentmem/config.yaml

hooks:
  pre_prompt:
    enabled: true
    timeout_ms: 5000
    max_tokens: 2000
    
  post_session:
    enabled: true
    auto_extract: true
    auto_sync: true
    
  post_compact:
    enabled: true
    show_reminder: true
```

---

## 8. Memory Extraction

### 8.1 Extraction Process

```
Session Transcript
       │
       ▼
┌──────────────────────────────────────┐
│         Memory Extractor             │
│                                      │
│  1. Parse transcript                 │
│  2. Identify "surprise moments"      │
│  3. Classify by memory type          │
│  4. Extract entities                 │
│  5. Generate embeddings              │
│  6. Deduplicate against existing     │
│  7. Store new memories               │
│                                      │
└──────────────────────────────────────┘
       │
       ▼
New Memories in DB
```

### 8.2 Extraction Triggers (What to Look For)

| Trigger | Memory Type | Example |
|---------|-------------|---------|
| "No, do X instead" | correction | User corrects agent behavior |
| "The URL is..." | infrastructure | User provides endpoint |
| "Use the script at..." | tool | User mentions existing tool |
| "Don't modify..." | protected | User protects a file |
| "We decided to..." | decision | Explicit decision |
| "That broke because..." | gotcha | Something failed |
| Positive reaction | insight | User enthusiasm |
| Repeated pattern | pattern | Same action 3+ times |

### 8.3 Extraction Prompt

```markdown
# Memory Extraction

Analyze this session transcript and extract memories.

## Memory Types (extract these)

1. **correction**: User corrected agent behavior
   - Look for: "no", "don't", "instead", "actually", "wrong"
   - High confidence (90+)

2. **infrastructure**: URLs, endpoints, credentials locations
   - Look for: URLs, "the API is", "database", ".env"
   - High confidence (85+)

3. **tool**: Existing scripts or utilities
   - Look for: "use the script", "there's a utility", file paths
   - High confidence (85+)

4. **protected**: Files not to modify
   - Look for: "don't modify", "leave alone", "working"
   - Critical (95+)

5. **decision**: Architectural choices
   - Look for: "we decided", "chose", "because"
   - Include reasoning
   - Medium confidence (70+)

6. **gotcha**: Things that broke or surprised
   - Look for: errors, "broke", "doesn't work", "careful"
   - Medium confidence (75+)

7. **pattern**: Repeated behaviors
   - Look for: same action done 3+ times
   - Lower confidence (60+)

8. **insight**: Non-obvious discoveries
   - Look for: "interesting", "turns out", discoveries
   - Lower confidence (60+)

## Output Format

```json
{
  "memories": [
    {
      "type": "correction",
      "title": "Brief title",
      "content": "Full description",
      "source_chunk": "Exact transcript excerpt",
      "confidence": 90,
      "entities": [{"type": "person", "name": "Amy"}],
      "reasoning": "Why this was extracted"
    }
  ]
}
```

## Transcript

{transcript}
```

### 8.4 Deduplication

Before adding a new memory:

1. Search existing memories by embedding similarity
2. If similarity > 0.9, update existing instead of creating new
3. Increment `times_observed` on existing
4. Update `last_observed_at`

---

## 9. Retrieval System

### 9.1 Retrieval Flow

```
User Query: "Fix the authentication bug"
                    │
                    ▼
         ┌─────────────────────┐
         │  Entity Extraction  │
         │  - "authentication" │
         └──────────┬──────────┘
                    │
        ┌───────────┴───────────┐
        ▼                       ▼
┌───────────────┐       ┌───────────────┐
│ Entity Match  │       │Semantic Search│
│               │       │               │
│ Filter by     │       │ Embed query   │
│ entity slug   │       │ Vector search │
└───────┬───────┘       └───────┬───────┘
        │                       │
        └───────────┬───────────┘
                    ▼
         ┌─────────────────────┐
         │    Re-Ranking       │
         │                     │
         │ - Similarity (60%)  │
         │ - Recency (15%)     │
         │ - Confidence (15%)  │
         │ - Type boost (10%)  │
         └──────────┬──────────┘
                    │
                    ▼
         ┌─────────────────────┐
         │   Top N Results     │
         │   (default: 5)      │
         └─────────────────────┘
```

### 9.2 Re-Ranking Algorithm

```python
def calculate_score(memory, query, base_similarity):
    score = 0.0
    
    # Base similarity (60%)
    score += base_similarity * 0.60
    
    # Recency boost (15%) - exponential decay over 4 weeks
    days_old = (now - memory.last_observed_at).days
    recency = math.exp(-days_old / 28)
    score += recency * 0.15
    
    # Confidence (15%)
    score += (memory.confidence / 100) * 0.15
    
    # Type boost (10%) - critical types get boost
    if memory.type in ['protected', 'correction', 'infrastructure']:
        score += 0.10
    elif memory.type in ['tool', 'decision']:
        score += 0.05
    
    # Query-aware type boost (+15% bonus)
    if query_matches_type(query, memory.type):
        score *= 1.15
    
    # Feedback adjustment
    if memory.positive_feedback > memory.negative_feedback:
        score *= 1.10
    elif memory.negative_feedback > 3:
        score *= 0.70
    
    return min(score, 1.0)

def query_matches_type(query, memory_type):
    keywords = {
        'decision': ['decided', 'chose', 'choice', 'why'],
        'correction': ['mistake', 'wrong', 'error', 'don\'t'],
        'infrastructure': ['url', 'endpoint', 'api', 'database'],
        'tool': ['script', 'utility', 'use', 'run'],
        'protected': ['modify', 'change', 'edit'],
        'gotcha': ['broke', 'failed', 'careful', 'issue']
    }
    return any(kw in query.lower() for kw in keywords.get(memory_type, []))
```

### 9.3 Always-Include Rules

Some memories should always be included regardless of query:

1. **Protected files** — Always show (critical for safety)
2. **Active corrections** — Recent corrections always relevant
3. **Current task context** — If working on a task, show related memories

---

## 10. Storage & Sync

### 10.1 Local Storage

```
.agentmem/
├── agentmem.db              # SQLite database (gitignored)
├── agentmem.jsonl           # Exported data (git-tracked)
├── embeddings.db            # Embedding cache (gitignored)
├── config.yaml              # Configuration
├── hooks/                   # Hook scripts
│   ├── pre-prompt.js
│   └── post-session.js
└── .gitignore
```

### 10.2 Git Sync Flow

```
Local Changes
      │
      ▼
am sync
      │
      ├── Export DB → JSONL
      │
      ├── git add .agentmem/agentmem.jsonl
      │
      ├── git commit -m "agentmem: sync"
      │
      └── (optional) git push


Remote Changes
      │
      ▼
git pull
      │
      ▼
am import (auto-triggered by git hook)
      │
      ▼
Merge JSONL → DB
      │
      ▼
Regenerate embeddings (if missing)
```

### 10.3 Conflict Resolution

When JSONL has conflicts:

1. Parse both versions
2. Use timestamp (`_ts`) to determine winner
3. For memories: later observation wins
4. For tasks: merge changes, latest status wins
5. For protected: union of both sets

---

## 11. Integration with Agents

### 11.1 Claude Code Integration

```bash
# Install hooks
am hook install claude-code

# This adds to .claude/settings.json:
{
  "hooks": {
    "UserPromptSubmit": [".agentmem/hooks/pre-prompt.js"],
    "SessionEnd": [".agentmem/hooks/post-session.js"]
  }
}

# And adds to CLAUDE.md:
## AgentMem

This project uses AgentMem for persistent context.
- Run `am task ready` to see current tasks
- Run `am context` to get relevant memories
- Protected files require approval before modification
```

### 11.2 Cursor Integration

```bash
am hook install cursor

# Adds to .cursor/settings.json
```

### 11.3 Generic Integration (Manual)

```markdown
# Add to CLAUDE.md / AGENTS.md

## AgentMem Integration

Before starting work:
1. Run `am task ready --json` to see unblocked tasks
2. Run `am context --query "<your task>" --json` to get relevant context

During work:
- Check `am protected list` before modifying files
- Run `am tool list` to see available utilities
- Use `am task update <id> --status in_progress` when starting

When you learn something:
- `am protect <file>` - Mark file as protected
- `am tool <path> <desc>` - Register a utility
- `am mem add <type> <title>` - Add a memory

Session end:
- `am task close <id>` for completed tasks
- `am sync` to save to git
```

---

## 12. Technical Requirements

### 12.1 Core Requirements

| Component | Requirement |
|-----------|-------------|
| Language | Rust (chosen for performance and availability) |
| Database | SQLite 3.35+ (with JSON and math functions) |
| Embeddings | Optional: Ollama, Gemini API, or OpenAI API |
| Git | Git 2.20+ |
| OS | macOS, Linux, Windows (WSL recommended) |

### 12.2 Embedding Options

| Provider | Model | Dimensions | Cost | Latency |
|----------|-------|------------|------|---------|
| Ollama (local) | mxbai-embed-large | 1024 | Free | ~100ms |
| Ollama (local) | nomic-embed-text | 768 | Free | ~50ms |
| Gemini | text-embedding-004 | 768 | $0.00001/1K | ~200ms |
| OpenAI | text-embedding-3-small | 1536 | $0.00002/1K | ~200ms |

### 12.3 Performance Targets

| Operation | Target |
|-----------|--------|
| `am context` | < 500ms |
| `am task ready` | < 100ms |
| `am mem search` | < 300ms |
| `am sync` | < 2s |
| Hook injection | < 1s total |

---

## 13. Implementation Phases

### Phase 1: Core CLI (2-3 weeks)

**Goal**: Basic task and memory management

- [ ] Project structure and build system
- [ ] SQLite database with schema
- [ ] `am init` command
- [ ] Task CRUD commands
- [ ] Memory CRUD commands
- [ ] `am protect`, `am tool` shortcuts
- [ ] JSONL export/import
- [ ] `am sync` with git
- [ ] Basic `am context` (no embeddings)

**Deliverable**: Working CLI that can track tasks and memories manually.

### Phase 2: Hooks & Injection (1-2 weeks)

**Goal**: Automatic context injection

- [ ] Hook system architecture
- [ ] Claude Code pre-prompt hook
- [ ] Context formatting (markdown)
- [ ] Protected files always-include
- [ ] Post-session hook (stub)
- [ ] `am hook install` command

**Deliverable**: Context injected automatically on each prompt.

### Phase 3: Embeddings & Semantic Search (2 weeks)

**Goal**: Intelligent retrieval

- [ ] Embedding service abstraction
- [ ] Ollama integration
- [ ] Gemini integration
- [ ] Vector storage in SQLite
- [ ] Semantic search implementation
- [ ] Re-ranking algorithm
- [ ] `am mem search` with embeddings
- [ ] Hybrid retrieval (entity + semantic)

**Deliverable**: Relevant memories retrieved per query.

### Phase 4: Automatic Extraction (2 weeks)

**Goal**: Learn from sessions automatically

- [ ] Transcript parsing
- [ ] Extraction prompt engineering
- [ ] LLM integration for extraction
- [ ] Deduplication logic
- [ ] `am extract` command
- [ ] Post-session hook completion
- [ ] Entity extraction and resolution

**Deliverable**: Memories extracted automatically from sessions.

### Phase 5: Polish & Ecosystem (1-2 weeks)

**Goal**: Production ready

- [ ] Cursor integration
- [ ] Windsurf integration
- [ ] `am doctor` health checks
- [ ] `am stats` analytics
- [ ] Documentation
- [ ] Installation scripts
- [ ] Homebrew formula
- [ ] npm package

**Deliverable**: Ready for public use.

---

## 14. Success Metrics

### 14.1 Quantitative

| Metric | Target |
|--------|--------|
| Context injection latency | < 1s |
| Memory retrieval precision | > 80% relevant |
| Extraction accuracy | > 70% useful memories |
| Token overhead per query | < 2000 tokens |
| Compaction survival | 100% (context restored) |

### 14.2 Qualitative

- Agent doesn't recreate existing utilities
- Agent doesn't modify protected files without asking
- Agent knows infrastructure details without being told
- Agent remembers decisions across sessions
- Agent doesn't repeat same mistakes

### 14.3 User Validation

After 2 weeks of use:
- "I no longer have to repeat myself"
- "Agent remembers my project structure"
- "Fewer destructive changes"
- "Agent uses existing scripts"

---

## 15. Open Questions

### 15.1 Technical

1. **Go vs Rust**: Chosen Rust for performance and system availability.
2. **Vector storage**: SQLite blob vs dedicated vector DB (for scale)?
3. **Embedding model**: Default to local (Ollama) or cloud (Gemini)?
4. **Hook format**: JS for Claude Code compatibility or Python for flexibility?

### 15.2 Product

1. **Memory decay**: Should old memories fade? How?
2. **Feedback loop**: How to collect thumbs up/down in CLI context?
3. **Multi-project**: One DB per project or global?
4. **Team sharing**: How to share entity definitions across team?

### 15.3 Integration

1. **Claude Code hooks**: What events are available?
2. **Cursor hooks**: Different hook system?
3. **Transcript format**: Standardized or agent-specific parsing?

---

## Appendix A: Example Session

```bash
# Initialize in project
$ am init --embedding ollama
✓ Created .agentmem/
✓ Initialized database
✓ Using Ollama for embeddings (mxbai-embed-large)
✓ Installed git hooks

# Add initial context
$ am protect src/prompts/system.md "Working production prompt"
✓ Protected: src/prompts/system.md

$ am protect prisma/schema.prisma "Database schema"
✓ Protected: prisma/schema.prisma

$ am tool scripts/translate.ts "Translation via Gemini API" "npx ts-node scripts/translate.ts en vi"
✓ Registered tool: translate

$ am infra "Railway API" "https://api.farmerchat.railway.app"
✓ Added infrastructure: Railway API

$ am decide "Use Prisma ORM" "Type safety, works with Railway, good migrations"
✓ Added decision: Use Prisma ORM

# Create a task
$ am task create "Fix authentication bug" -p 1 -t bug -l auth,backend
✓ Created: am-a1b2 "Fix authentication bug"

# Check ready tasks
$ am task ready
[P1] am-a1b2: Fix authentication bug (open)

# Get context for current work
$ am context --query "fix authentication" --format markdown

## ⚠️ Protected Files
- `src/prompts/system.md` — Working production prompt
- `prisma/schema.prisma` — Database schema

## Current Tasks
- [P1] am-a1b2: Fix authentication bug (open)

## Relevant Context
- [infrastructure] Railway API: https://api.farmerchat.railway.app
- [decision] Use Prisma ORM: Type safety, works with Railway

## Available Tools
- `scripts/translate.ts` — Translation via Gemini API

# After session, sync to git
$ am sync
✓ Exported 5 tasks, 8 memories
✓ Committed: agentmem: sync (a1b2c3d)
```

---

## Appendix B: Comparison with Existing Tools

| Feature | Beads | Memory Lane | AgentMem |
|---------|-------|-------------|----------|
| Task tracking | ✅ Full | ❌ None | ✅ Full |
| Semantic memory | ❌ None | ✅ Full | ✅ Full |
| Auto-extraction | ❌ Manual | ✅ Full | ✅ Full |
| Protected files | ❌ None | ❌ None | ✅ Built-in |
| Tool registry | ❌ None | ❌ None | ✅ Built-in |
| Hook injection | ❌ Query-based | ✅ Per-prompt | ✅ Per-prompt |
| Git sync | ✅ JSONL | ❌ PostgreSQL | ✅ JSONL |
| External deps | None | PostgreSQL, Ollama | SQLite only |
| Embedding | ❌ None | ✅ Required | ⚡ Optional |

---

## Appendix C: File Structure (Rust)

```
agentmem/
├── src/
│   ├── main.rs                  # CLI entry point
│   ├── db/
│   │   ├── mod.rs
│   │   ├── sqlite.rs            # SQLite operations
│   │   ├── migrations.rs        # Schema migrations
│   │   └── models.rs            # Data models
│   ├── tasks/
│   │   ├── mod.rs
│   │   ├── service.rs           # Task business logic
│   │   └── ready.rs             # Ready work detection
│   ├── memory/
│   │   ├── mod.rs
│   │   ├── service.rs           # Memory business logic
│   │   ├── types.rs             # Memory type definitions
│   │   └── extraction.rs        # Transcript extraction
│   ├── retrieval/
│   │   ├── mod.rs
│   │   ├── context.rs           # Context building
│   │   ├── search.rs            # Semantic search
│   │   ├── ranking.rs           # Re-ranking algorithm
│   │   └── entities.rs          # Entity resolution
│   ├── embedding/
│   │   ├── mod.rs
│   │   ├── service.rs           # Embedding abstraction
│   │   ├── ollama.rs            # Ollama provider
│   │   ├── gemini.rs            # Gemini provider
│   │   └── openai.rs            # OpenAI provider
│   ├── sync/
│   │   ├── mod.rs
│   │   ├── export.rs            # JSONL export
│   │   ├── import.rs            # JSONL import
│   │   └── git.rs               # Git operations
│   ├── hooks/
│   │   ├── mod.rs
│   │   ├── manager.rs           # Hook management
│   │   └── templates/           # Hook templates
│   │       ├── claude-code.js
│   │       └── cursor.js
│   └── config/
│       ├── mod.rs
│       └── config.rs            # Configuration management
├── scripts/
│   ├── install.sh               # Installation script
│   └── bump-version.sh          # Version management
├── docs/
│   ├── INSTALLING.md
│   ├── QUICKSTART.md
│   └── HOOKS.md
├── Cargo.toml
├── Makefile
└── README.md
```

---

*End of PRD*

