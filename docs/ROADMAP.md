# AgentMem Roadmap

> **Vision**: A dependable memory system for AI-assisted software development.

## Current Status: v0.2 Beta

Phases 1-4 are complete. The core CLI works with Claude Code integration, semantic search, and memory extraction.

---

## Completed Phases

### Phase 1: Core Foundation
- [x] CLI tool (`am`) with clap-based commands
- [x] SQLite storage (tasks, memories, protected files, tools)
- [x] Git-based sync (JSONL export/import)
- [x] YAML configuration

### Phase 2: Claude Code Integration
- [x] Pre-prompt hook (context injection)
- [x] Post-session hook (auto-sync)
- [x] Protected file warnings
- [x] Markdown/JSON context formatting
- [x] `.cjs` hook format for CommonJS compatibility

### Phase 3: Semantic Search
- [x] OpenAI embeddings (text-embedding-3-small)
- [x] Qdrant vector store integration
- [x] Similarity-based memory retrieval
- [x] Graceful fallback when Qdrant unavailable

### Phase 4: Memory Extraction
- [x] GPT-4o transcript analysis
- [x] 8 memory types (correction, decision, gotcha, etc.)
- [x] Automatic deduplication via semantic similarity
- [x] JSONL and plain text transcript support

---

## In Progress

### Phase 5: One-Command Installation

Seamless setup for new users.

**Features:**
- [ ] Install script for macOS/Linux (`curl | bash`)
- [ ] Docker detection and auto-install prompt
- [ ] Qdrant container auto-start on `am init`
- [ ] OpenAI key setup wizard
- [ ] Credentials stored at `~/.agentmem/credentials`
- [ ] `am doctor` health check (partially done)
- [ ] Uninstall script

---

## Planned

### Phase 6: Multi-Agent Support

- [ ] Gemini CLI hooks
- [ ] Codex CLI hooks
- [ ] Cursor hooks
- [ ] Agent-specific context formatting
- [ ] Auto-detection of installed agents

### Phase 7: Cloud Dashboard

API-based architecture for web dashboard and team sync.

- [ ] PostgreSQL cloud database (Railway)
- [ ] REST API server
- [ ] User authentication (API keys)
- [ ] Web dashboard (Next.js + ShadCN)
- [ ] Project management
- [ ] Token analytics
- [ ] Decision timeline

See [PHASE7_ARCHITECTURE.md](PHASE7_ARCHITECTURE.md) for technical details.

### Phase 8: Team Collaboration

- [ ] Git remote sync (`am sync --push/pull`)
- [ ] Team namespaces (personal vs shared)
- [ ] User attribution on memories
- [ ] Conflict resolution
- [ ] Memory visibility controls

### Phase 9: Extended Memory Types

SDLC-focused memory types:

**Architecture:** architecture, api-contract, data-model, requirement
**Development:** correction, decision, pattern, gotcha (existing)
**Code Review:** review-feedback, style-guide, security
**QA:** test-case, bug, regression
**Deployment:** infrastructure (existing), env-config, runbook

### Phase 10: Documentation Site

- [ ] Docusaurus or Nextra-based site
- [ ] Quick start guide
- [ ] User guides by role
- [ ] CLI reference
- [ ] Integration guides

---

## Success Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Setup time | < 5 minutes | ~3 min |
| Context retrieval | < 500ms | ~200ms |
| Agent support | 4 agents | 1 (Claude Code) |
| Semantic search | Working | Yes |
| Memory extraction | Working | Yes |

---

## Release Plan

| Version | Status | Focus |
|---------|--------|-------|
| v0.1 | Done | Core CLI + basic hooks |
| v0.2 | Current | Semantic search + extraction |
| v0.3 | Next | One-command install |
| v0.4 | Planned | Multi-agent support |
| v0.5 | Planned | Cloud dashboard |
| v1.0 | Future | Full documentation + polish |
