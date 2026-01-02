# AgentMem Context

This project uses AgentMem for persistent memory across sessions.

## Quick Commands
- `am context` - Get relevant memories for current task
- `am mem add <type> <title>` - Add a memory (types: decision, correction, gotcha, pattern)
- `am protect <file>` - Mark file as protected (requires approval to modify)
- `am task ready` - See unblocked tasks
- `am sync` - Sync memories to git

## Important
Before modifying any file, check if it's protected with `am context`.
Protected files require explicit user approval before changes.

## Memory Types
- **correction**: When user corrects a mistake
- **decision**: Architectural/technical choices
- **gotcha**: Things that broke or surprised
- **pattern**: Repeated behaviors or preferences
- **infrastructure**: URLs, endpoints, configs
