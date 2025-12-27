# Phase 7: Web Dashboard Architecture

## Overview

API-based architecture with user identity for future multi-user scalability.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              DEVELOPER MACHINE                          │
│                                                                         │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                 │
│  │ Claude Code │    │   Cursor    │    │ Gemini CLI  │                 │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘                 │
│         │                  │                  │                         │
│         └──────────────────┼──────────────────┘                         │
│                            │                                            │
│                            ▼                                            │
│                   ┌─────────────────┐                                   │
│                   │    am CLI       │                                   │
│                   │  (with hooks)   │                                   │
│                   └────────┬────────┘                                   │
│                            │                                            │
└────────────────────────────┼────────────────────────────────────────────┘
                             │
                             │ HTTPS (API calls)
                             ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           RAILWAY.COM                                   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      API SERVER                                  │   │
│  │                   (Rust/Axum or Node)                           │   │
│  │                                                                  │   │
│  │  POST /api/memories      - Create memory                        │   │
│  │  GET  /api/memories      - List memories (with filters)         │   │
│  │  POST /api/sessions      - Start session                        │   │
│  │  PUT  /api/sessions/:id  - End session (with token stats)       │   │
│  │  GET  /api/projects      - List user's projects                 │   │
│  │  GET  /api/analytics     - Token usage, decision timeline       │   │
│  │  GET  /api/context       - Get context for prompt injection     │   │
│  │                                                                  │   │
│  └──────────────────────────────┬──────────────────────────────────┘   │
│                                 │                                       │
│                                 ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      PostgreSQL                                  │   │
│  │                                                                  │   │
│  │  users, projects, sessions, memories, protected_files, etc.     │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      WEB DASHBOARD                               │   │
│  │                   (Next.js + ShadCN)                            │   │
│  │                                                                  │   │
│  │  - Project overview                                             │   │
│  │  - Memory browser (global + per-project)                        │   │
│  │  - Token analytics & cost tracking                              │   │
│  │  - Decision timeline with outcomes                              │   │
│  │  - Agent/model breakdown                                        │   │
│  │                                                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Database Schema (PostgreSQL)

```sql
-- Users (simple for now, expandable for teams later)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE,
    name VARCHAR(255),
    api_key VARCHAR(64) UNIQUE NOT NULL,  -- For CLI authentication
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Projects registry
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    name VARCHAR(255) NOT NULL,           -- "agentmem"
    path VARCHAR(1024),                   -- "/Users/eagle/agentmem" (for reference)
    machine_id VARCHAR(255),              -- Identify which machine
    created_at TIMESTAMP DEFAULT NOW(),
    last_active_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(user_id, name, machine_id)
);

-- Sessions (each agent invocation)
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    project_id UUID REFERENCES projects(id),
    agent VARCHAR(50) NOT NULL,           -- "claude-code", "cursor", etc.
    model VARCHAR(100),                   -- "claude-sonnet-4-20250514", "gpt-4o"
    started_at TIMESTAMP DEFAULT NOW(),
    ended_at TIMESTAMP,
    tokens_in INTEGER DEFAULT 0,
    tokens_out INTEGER DEFAULT 0,
    cost_usd DECIMAL(10, 6) DEFAULT 0,
    status VARCHAR(20) DEFAULT 'active'   -- active, completed, error
);

-- Memories (core data)
CREATE TABLE memories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    project_id UUID REFERENCES projects(id),  -- NULL for global scope
    session_id UUID REFERENCES sessions(id),
    scope VARCHAR(20) DEFAULT 'project',      -- "global" or "project"
    memory_type VARCHAR(50) NOT NULL,         -- decision, correction, gotcha, etc.
    title VARCHAR(500) NOT NULL,
    content TEXT,
    agent VARCHAR(50),                        -- Which agent created this
    model VARCHAR(100),                       -- Which model
    outcome VARCHAR(20) DEFAULT 'unknown',    -- success, failed, unknown
    confidence INTEGER DEFAULT 70,
    times_observed INTEGER DEFAULT 1,
    source TEXT,                              -- Transcript excerpt
    embedding_id VARCHAR(100),                -- Qdrant point ID
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    last_observed_at TIMESTAMP DEFAULT NOW()
);

-- Protected files (per project)
CREATE TABLE protected_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    project_id UUID REFERENCES projects(id),
    pattern VARCHAR(500) NOT NULL,
    reason TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Tasks (per project)
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    project_id UUID REFERENCES projects(id),
    title VARCHAR(500) NOT NULL,
    description TEXT,
    status VARCHAR(50) DEFAULT 'open',
    priority INTEGER DEFAULT 2,
    task_type VARCHAR(50) DEFAULT 'task',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Indexes for common queries
CREATE INDEX idx_memories_user_project ON memories(user_id, project_id);
CREATE INDEX idx_memories_user_scope ON memories(user_id, scope);
CREATE INDEX idx_sessions_user_project ON sessions(user_id, project_id);
CREATE INDEX idx_sessions_started ON sessions(started_at DESC);
```

## Authentication Flow

```
1. First time setup:
   $ am auth login
   > Enter email: eagle@example.com
   > Check your email for verification link...
   > ✓ Logged in! API key saved to ~/.agentmem/credentials

2. Or use API key directly:
   $ am auth login --api-key am_xxxxxxxxxxxx
   > ✓ Authenticated!

3. CLI stores credentials:
   ~/.agentmem/credentials:
   API_KEY=am_xxxxxxxxxxxx
   USER_ID=uuid-here
   EMAIL=eagle@example.com

4. All API calls include:
   Authorization: Bearer am_xxxxxxxxxxxx
```

## API Endpoints

### Authentication
```
POST /api/auth/register     - Create account, get API key
POST /api/auth/login        - Login with email (sends magic link)
GET  /api/auth/verify       - Verify magic link token
GET  /api/auth/me           - Get current user info
```

### Projects
```
GET    /api/projects                - List all projects
POST   /api/projects                - Register a project
GET    /api/projects/:id            - Get project details
DELETE /api/projects/:id            - Unregister project
```

### Sessions
```
POST   /api/sessions                - Start a session
PUT    /api/sessions/:id            - Update session (end, add tokens)
GET    /api/sessions                - List sessions (with filters)
GET    /api/sessions/:id            - Get session details
```

### Memories
```
GET    /api/memories                - List memories (filters: project, scope, type)
POST   /api/memories                - Create memory
GET    /api/memories/:id            - Get memory details
PUT    /api/memories/:id            - Update memory (outcome, content)
DELETE /api/memories/:id            - Delete memory
POST   /api/memories/:id/promote    - Promote to global scope
```

### Context (for hooks)
```
GET    /api/context                 - Get context for current project
        ?project=name
        &query=search-term
        &limit=10
```

### Analytics
```
GET    /api/analytics/tokens        - Token usage over time
        ?period=7d|30d|90d
        &project=id (optional)

GET    /api/analytics/decisions     - Decision timeline
        ?period=7d
        &outcome=success|failed|all

GET    /api/analytics/agents        - Agent/model breakdown
        ?period=30d
```

## CLI Changes

```bash
# Authentication
am auth login                    # Interactive login
am auth login --api-key KEY      # Direct API key
am auth logout                   # Clear credentials
am auth status                   # Show current user

# All commands now sync to cloud
am mem add decision "Use PostgreSQL"     # Syncs to API
am mem add decision "Use pnpm" --global  # Global scope
am mem promote <id>                      # Promote to global

# Context retrieval (from API)
am context --query "database"            # Fetches from API

# Project management
am projects list                         # All registered projects
am projects current                      # Show current project

# Offline mode (fallback to local SQLite)
am --offline mem add ...                 # Local only, sync later
am sync                                  # Push local changes to cloud
```

## Dashboard Pages

```
/                       - Dashboard home (projects, recent activity)
/projects               - All projects grid
/projects/:id           - Project detail (memories, tasks, protected)
/memories               - All memories browser (filter by scope, type)
/memories/:id           - Memory detail
/analytics              - Token usage, cost, agent breakdown
/analytics/timeline     - Decision timeline with outcomes
/settings               - User settings, API keys, preferences
```

## Tech Stack

| Component | Technology |
|-----------|------------|
| API Server | Rust (Axum) or Node.js (Express/Fastify) |
| Database | PostgreSQL (Railway managed) |
| Dashboard | Next.js 14 + App Router |
| UI Components | ShadCN/ui |
| Styling | Tailwind CSS (jet black/white theme) |
| Charts | Recharts |
| Icons | Lucide |
| Auth | API keys (simple), Magic links (optional) |
| Hosting | Railway.com |

## Implementation Order

1. **Database & API Server** (Rust or Node)
   - PostgreSQL schema
   - Core CRUD endpoints
   - API key authentication

2. **CLI Updates**
   - Add `am auth` commands
   - Update all commands to use API
   - Offline fallback mode

3. **Dashboard MVP**
   - Projects list
   - Memories browser
   - Basic analytics

4. **Dashboard Full**
   - Token analytics charts
   - Decision timeline
   - Agent/model breakdown
   - Settings page

## Environment Variables

```bash
# API Server
DATABASE_URL=postgres://...
JWT_SECRET=xxx
API_KEY_PREFIX=am_

# Dashboard
NEXT_PUBLIC_API_URL=https://api.agentmem.dev
```

## Deployment (Railway)

```
railway/
├── api/                 # Rust or Node API
│   ├── Dockerfile
│   └── railway.toml
├── dashboard/           # Next.js app
│   ├── Dockerfile
│   └── railway.toml
└── docker-compose.yml   # Local development
```
