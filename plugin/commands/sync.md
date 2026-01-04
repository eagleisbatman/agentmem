---
description: Sync AgentMem data to git for persistence and sharing
---

# Sync Command

Export AgentMem data to JSONL and commit to git.

## Usage

Run the command:
```bash
am sync
```

Or to also push to remote:
```bash
am sync --push
```

## What It Does

1. Exports all data (tasks, memories, protected files, tools) to `.agentmem/agentmem.jsonl`
2. Commits the JSONL file to git
3. Optionally pushes to remote

## When to Use

- At the end of a session
- After adding important memories
- Before switching machines
- When the user explicitly asks

After running, report the sync status.
