---
description: Show AgentMem system status and health
---

# Status Command

Check AgentMem system health and configuration.

## Usage

Run the command:
```bash
am doctor --json
```

## What It Checks

- AgentMem initialization status
- Database health
- Docker status
- Qdrant vector database status
- OpenAI API key configuration
- Installed hooks

## Output

Parse the JSON output and present a human-readable status:

- OK items in green/positive
- Issues in red/warning with remediation steps

## When to Use

- When something seems wrong
- To verify setup is complete
- When debugging hook issues
