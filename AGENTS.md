# AgentMem Integration

This project uses AgentMem for persistent memory. Before starting work:

1. Run `am context` to see relevant memories and protected files
2. Protected files require explicit approval before modification
3. After significant changes, memories are automatically extracted

## Commands
- `am context` - Get relevant context
- `am mem add <type> <title>` - Add a memory
- `am protect <file>` - Mark file as protected
- `am sync` - Sync to git

## Memory Types
- decision: Architectural choices
- correction: Mistakes to avoid
- gotcha: Things that broke
- pattern: Repeated preferences
