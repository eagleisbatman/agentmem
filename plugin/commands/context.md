---
description: Show the current AgentMem context that would be injected
---

# Context Command

Display the current AgentMem context including memories, tasks, protected files, and tools.

## Usage

Run the command:
```bash
am context --format markdown
```

Or with a specific query:
```bash
am context --query "<topic>" --format markdown
```

## What It Shows

- **Protected Files**: Files requiring approval before modification
- **Current Tasks**: Ready/in-progress tasks with priorities
- **Relevant Memories**: Decisions, gotchas, patterns, etc.
- **Available Tools**: Registered scripts and utilities

## When to Use

- When the user wants to see what context is available
- To debug what memories are being retrieved
- To verify protected files are set correctly

Display the output in a readable format.
