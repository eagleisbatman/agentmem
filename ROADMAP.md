# AgentMem Roadmap

> **Vision**: A dependable memory system for AI-assisted software development, covering the complete lifecycle from architecture to deployment.

---

## Target Users

- **Developers** - Individual contributors using AI coding agents daily
- **Architects** - Technical leads designing systems across sessions
- **Product Managers** - Non-technical users leveraging AI for specs and planning
- **QA Engineers** - Testing workflows with AI assistance
- **DevOps** - Deployment and infrastructure automation

---

## Supported AI Agents (MVP)

| Agent | Platform | Status |
|-------|----------|--------|
| Claude Code | CLI | ✅ Supported |
| Gemini CLI | CLI | 🔲 Planned |
| Codex CLI | CLI | 🔲 Planned |
| Cursor | IDE | 🔲 Planned |

---

## Technology Stack

| Component | Technology | Notes |
|-----------|------------|-------|
| Storage | SQLite | Local, portable |
| Vector DB | Qdrant | Docker-based, auto-installed |
| Embeddings | OpenAI | text-embedding-3-small |
| Extraction | GPT-4o | Memory extraction from transcripts |
| Dashboard | ShadCN/UI | Jet black/white, line icons |
| Deployment | Railway.com | One-click deploy |

---

## Completed

### Phase 1: Core Foundation ✅
- CLI tool (`am`) with intuitive commands
- SQLite storage for tasks, memories, protected files, tools
- Git-based sync (JSONL export/import)
- YAML configuration

### Phase 2: Claude Code Integration ✅
- Pre-prompt hook (context injection)
- Post-session hook (auto-sync)
- Protected file warnings
- Markdown/JSON context formatting

### Phase 3: Semantic Search ✅
- OpenAI embeddings integration
- Qdrant vector store
- Similarity-based memory retrieval
- Graceful fallback when Qdrant unavailable

### Phase 4: Memory Extraction ✅
- GPT-4o transcript analysis
- 8 memory types (correction, decision, gotcha, etc.)
- Automatic deduplication
- JSONL and plain text support

---

## MVP Roadmap

### Phase 5: One-Command Installation
Seamless setup experience for new users.

```bash
# Install AgentMem globally
curl -sSL https://agentmem.dev/install.sh | bash

# Initialize in project (auto-installs Docker + Qdrant)
am init
```

**Features:**
- [ ] Install script for macOS/Linux
- [ ] Docker detection and auto-install prompt
- [ ] Qdrant container auto-start
- [ ] OpenAI key setup wizard
  - [ ] Read from `~/.agentmem/credentials`
  - [ ] Environment variable fallback
  - [ ] Interactive prompt if missing
- [ ] Health check (`am doctor`)
- [ ] Uninstall script

**User Experience:**
```
$ am init

🔍 Checking dependencies...
  ✓ Docker installed
  ✓ Qdrant container running
  ✗ OpenAI API key not found

📝 Enter your OpenAI API key: sk-...
  ✓ Key saved to ~/.agentmem/credentials

✨ AgentMem initialized! Run 'am hook install claude-code' to get started.
```

---

### Phase 6: Multi-Agent Support
Extend to all primary AI coding agents.

**Agents:**
- [ ] **Gemini CLI** - Google's AI assistant
- [ ] **Codex CLI** - OpenAI's coding agent
- [ ] **Cursor** - IDE with AI integration

**Features:**
- [ ] `am hook install gemini-cli`
- [ ] `am hook install codex-cli`
- [ ] `am hook install cursor`
- [ ] Agent-specific context formatting
- [ ] Unified transcript format across agents
- [ ] Auto-detection of installed agents

---

### Phase 7: Full SDLC Memory Types
Memories tailored to each development phase.

**Architecture & Design:**
- [ ] `architecture` - System design decisions
- [ ] `api-contract` - API specifications
- [ ] `data-model` - Database schemas
- [ ] `requirement` - Product requirements

**Development:**
- [ ] `correction` - Agent mistakes to avoid (existing)
- [ ] `decision` - Technical choices (existing)
- [ ] `pattern` - Code patterns and conventions
- [ ] `gotcha` - Pitfalls and edge cases (existing)

**Code Review:**
- [ ] `review-feedback` - PR review comments
- [ ] `style-guide` - Coding standards
- [ ] `security` - Security considerations

**QA & Testing:**
- [ ] `test-case` - Important test scenarios
- [ ] `bug` - Known issues and fixes
- [ ] `regression` - Things that broke before

**Deployment:**
- [ ] `infrastructure` - Deployment configs (existing)
- [ ] `env-config` - Environment variables
- [ ] `runbook` - Operational procedures

**Platform-Specific:**
- [ ] Mobile app memories (iOS, Android, React Native, Flutter)
- [ ] Web app memories (React, Vue, Next.js, etc.)
- [ ] Backend memories (Node, Python, Go, Rust, etc.)

---

### Phase 8: Team Collaboration
Share memories across team members.

**Features:**
- [ ] Git remote sync (`am sync --push`)
- [ ] Team namespaces (personal vs shared)
- [ ] User attribution on memories
- [ ] Conflict resolution
- [ ] Memory visibility controls (private/team/public)

**Workflow:**
```bash
# Developer A adds a memory
am mem add decision "Use PostgreSQL for main database" \
  --content "Chose Postgres over MySQL for JSON support"

# Sync to team
am sync --push

# Developer B pulls team memories
am sync --pull

# Context now includes team knowledge
am context --query "database"
```

---

### Phase 9: Web Dashboard
Beautiful, minimal interface for memory management.

**Design System:**
- [ ] ShadCN/UI components
- [ ] Jet black theme (default)
- [ ] Jet white theme (light mode)
- [ ] Lucide line icons
- [ ] Minimal, focused UI
- [ ] Responsive (desktop-first)

**Features:**
- [ ] `am serve` - Start local dashboard
- [ ] Memory browser with search
- [ ] Memory editor (add/edit/delete)
- [ ] Task board view
- [ ] Protected files manager
- [ ] Tools registry
- [ ] Entity relationship graph
- [ ] Activity timeline
- [ ] Team sync status

**Screens:**
1. **Dashboard** - Overview, recent memories, active tasks
2. **Memories** - Browse, search, filter by type
3. **Tasks** - Kanban-style task board
4. **Settings** - Config, agents, team sync

---

### Phase 10: Beautiful Documentation
Comprehensive docs for all user types.

**Documentation Site:**
- [ ] Docusaurus or Nextra-based
- [ ] Clean, readable design
- [ ] Dark/light mode
- [ ] Full-text search
- [ ] Copy-paste code blocks

**Content:**

**Getting Started:**
- [ ] Quick start (5 minutes)
- [ ] Installation guide
- [ ] First memory tutorial
- [ ] Video walkthrough

**User Guides:**
- [ ] For Developers
- [ ] For Product Managers
- [ ] For Architects
- [ ] For QA Engineers

**Workflow Guides:**
- [ ] Architecture design with AgentMem
- [ ] Code review workflow
- [ ] QA and testing workflow
- [ ] Deployment workflow
- [ ] Mobile app development
- [ ] Web app development

**Reference:**
- [ ] CLI command reference
- [ ] Memory types reference
- [ ] Configuration reference
- [ ] API reference (for integrations)

**Integration Guides:**
- [ ] Claude Code setup
- [ ] Gemini CLI setup
- [ ] Codex CLI setup
- [ ] Cursor setup

**Deployment:**
- [ ] Railway.com deployment
- [ ] Self-hosted setup
- [ ] Team setup

---

## Post-MVP Ideas

### Advanced Features
- Memory consolidation (merge similar memories)
- Memory decay (archive stale memories)
- Hybrid search (semantic + keyword + recency)
- Memory quality scoring

### Additional Integrations
- Notion import
- Confluence import
- Slack thread import
- GitHub issue/PR sync

### Enterprise (Future)
- SSO integration
- Audit logging
- Role-based access
- On-premise deployment

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Setup time | < 5 minutes |
| First memory | < 1 minute after setup |
| Context retrieval | < 500ms |
| Documentation completeness | 100% command coverage |
| Agent support | 4 agents (Claude, Gemini, Codex, Cursor) |

---

## Release Plan

| Version | Milestone | Focus |
|---------|-----------|-------|
| v0.1 | Alpha | Core CLI + Claude Code |
| v0.2 | Beta | One-command install + Qdrant |
| v0.3 | Beta | Multi-agent support |
| v0.4 | Beta | SDLC memory types |
| v0.5 | Beta | Team collaboration |
| v0.6 | RC | Web dashboard |
| v1.0 | Release | Documentation + Polish |

---

## Contributing

1. Pick an item from the roadmap
2. Open an issue to discuss approach
3. Submit a PR
4. Get featured in contributors list

## Feedback

Ideas? Issues? Open a GitHub discussion or reach out on Twitter.
