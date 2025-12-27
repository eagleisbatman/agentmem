# AgentMem Roadmap

Persistent memory for AI coding agents. This roadmap outlines completed work and planned features.

---

## Completed

### Phase 1: Core Foundation
- [x] CLI tool (`am`) with clap
- [x] SQLite storage for tasks, memories, protected files, tools
- [x] Task management (create, list, ready)
- [x] Memory CRUD operations
- [x] Git sync (export/import JSONL)
- [x] Configuration system (YAML)

### Phase 2: Hooks & Context Injection
- [x] Pre-prompt hook for Claude Code (UserPromptSubmit)
- [x] Post-session hook for sync (SessionEnd)
- [x] `am hook install claude-code` command
- [x] Context formatting (markdown/JSON)
- [x] Protected file warnings in context

### Phase 3: Embeddings & Semantic Search
- [x] EmbeddingProvider trait (pluggable providers)
- [x] OpenAI embeddings (text-embedding-3-small)
- [x] Qdrant vector store integration
- [x] `am mem search <query>` semantic search
- [x] Fallback to LIKE search when Qdrant unavailable

### Phase 4: Automatic Memory Extraction
- [x] GPT-4o integration for transcript analysis
- [x] `am extract --transcript <file>` command
- [x] 8 memory types (correction, decision, gotcha, etc.)
- [x] Automatic deduplication (>0.9 similarity threshold)
- [x] Post-session hook integration

---

## In Progress

### Phase 5: Entity Extraction & Linking
Extract and link entities (people, services, files) from memories for relationship queries.

**Features:**
- [ ] Entity extraction during memory creation
- [ ] Entity types: person, service, file, endpoint, database
- [ ] Relationship graph (memory <-> entity links)
- [ ] `am entity list` - list all entities
- [ ] `am entity search <name>` - find memories by entity
- [ ] Context enrichment with related entities

**Use cases:**
- "What did we decide about the auth service?"
- "Show memories related to database migrations"

---

## Planned

### Phase 6: Memory Lifecycle Management
Intelligent memory decay, consolidation, and relevance scoring.

**Features:**
- [ ] Relevance scoring (recency, recall frequency, confidence)
- [ ] Memory consolidation (merge similar memories)
- [ ] Automatic archival of stale memories
- [ ] `am mem prune` - archive low-relevance memories
- [ ] `am mem consolidate` - merge duplicates
- [ ] Configurable retention policies

### Phase 7: Additional Embedding Providers
Support local and alternative embedding providers.

**Providers:**
- [ ] Ollama (local, privacy-focused)
- [ ] Google Gemini
- [ ] Voyage AI
- [ ] Cohere

**Features:**
- [ ] `am init --embedding ollama --model nomic-embed-text`
- [ ] Automatic provider detection
- [ ] Embedding migration between providers

### Phase 8: Multi-Agent Support
Extend hook system to support additional AI coding agents.

**Agents:**
- [ ] Cursor (custom hook format)
- [ ] Windsurf
- [ ] Cody (Sourcegraph)
- [ ] Continue.dev
- [ ] Aider

**Features:**
- [ ] `am hook install cursor`
- [ ] Agent-specific context formatting
- [ ] Unified transcript format

### Phase 9: Team Collaboration
Share memories across team members.

**Features:**
- [ ] Team sync via git remote
- [ ] Conflict resolution for memories
- [ ] User attribution on memories
- [ ] `am sync --team` - push to shared remote
- [ ] Privacy controls (personal vs shared memories)
- [ ] Memory namespaces (project, team, personal)

### Phase 10: Web Dashboard
Visual interface for browsing and managing memories.

**Features:**
- [ ] Local web server (`am serve`)
- [ ] Memory browser with search
- [ ] Entity relationship graph visualization
- [ ] Task board view
- [ ] Memory editor
- [ ] Analytics (memory growth, recall frequency)

---

## Future Ideas

These are exploratory features that may be developed based on user feedback.

### Advanced Retrieval
- Hybrid search (semantic + keyword + recency)
- Query rewriting for better recall
- Multi-hop reasoning over memories

### IDE Extensions
- VS Code extension for inline context
- JetBrains plugin
- Neovim integration

### Memory Sources
- Import from Notion, Confluence, Slack
- GitHub issue/PR integration
- Meeting transcript import

### AI Improvements
- Fine-tuned extraction model
- Self-improving prompts based on feedback
- Memory quality scoring

### Enterprise Features
- SSO integration
- Audit logging
- Role-based access control
- On-premise deployment

---

## Contributing

Want to help? Pick an item from the roadmap and open a PR. For major features, open an issue first to discuss the approach.

## Feedback

Have ideas for the roadmap? Open an issue or discussion on GitHub.
