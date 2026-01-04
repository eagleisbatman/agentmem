---
name: memory-persistence
description: Automatically persist session state and extract learnings. Use when you notice decisions, corrections, gotchas, or at session end to preserve knowledge.
---

# Memory Persistence Skill

**CRITICAL**: You MUST use AgentMem to save learnings during every session. Do NOT wait until the end - save as you go. If you complete work without saving memories, knowledge is LOST.

## MANDATORY Save Points

You MUST run `am mem add` immediately after ANY of these events:

### 1. After Completing Any Feature or Fix
When you finish implementing something:
```bash
am mem add decision "<what was implemented>" --content "<key design choices and why>"
```

### 2. After Fixing Any Bug
When you fix an issue:
```bash
am mem add gotcha "<what broke>" --content "<root cause and solution>"
```

### 3. When User Corrects You
Phrases like "no", "actually", "that's wrong", "use X instead":
```bash
am mem add correction "<what you learned>" --content "<correct approach>"
```

### 4. After Architecture/Design Decisions
Technology choices, patterns, conventions:
```bash
am mem add decision "<decision>" --content "<reasoning and trade-offs>"
```

### 5. After Discovering Codebase Patterns
When you learn how the codebase works:
```bash
am mem add pattern "<pattern name>" --content "<how it works and where>"
```

### 6. After Finding Non-Obvious Information
API endpoints, env vars, config, credentials location:
```bash
am mem add infrastructure "<what>" --content "<details>"
```

## Periodic Checkpoints

**Every 5-10 messages**, pause and ask yourself:
- Did I learn anything worth saving?
- Did I make any decisions?
- Did anything break that I fixed?
- Did the user teach me something?

If yes to ANY, save immediately.

## Batch Save Checklist

Before responding to a user message that wraps up a piece of work, run through this checklist:

```
[ ] Decisions made this session?     → am mem add decision ...
[ ] Bugs fixed?                      → am mem add gotcha ...
[ ] User corrections?                → am mem add correction ...
[ ] New patterns discovered?         → am mem add pattern ...
[ ] Infrastructure learned?          → am mem add infrastructure ...
[ ] Tools/scripts discovered?        → am tool ...
[ ] Files to protect?                → am protect ...
```

## Memory Types Reference

| Type | When to Use |
|------|-------------|
| `decision` | Architecture choices, technology picks, design patterns |
| `correction` | User corrected your understanding or approach |
| `gotcha` | Something broke unexpectedly, non-obvious fix |
| `pattern` | Recurring code pattern in this codebase |
| `infrastructure` | API endpoints, env vars, URLs, credentials |
| `insight` | General learning that doesn't fit other categories |

## Anti-Patterns (DO NOT)

- DO NOT wait until session end to save
- DO NOT assume you'll remember next session
- DO NOT skip saving because "it's obvious"
- DO NOT save trivial/obvious facts (e.g., "React uses JSX")
- DO NOT duplicate existing memories (check with `am mem list` first)

## Session End Behavior

When session is ending or you detect compaction warnings:

1. **IMMEDIATELY** review conversation for unsaved learnings
2. Batch save anything missed
3. Run `am sync` to persist to git

## Example: Complete Feature Flow

```
1. User: "Add user authentication"

2. You implement it, making decisions along the way

3. BEFORE responding "Done!", run:
   am mem add decision "JWT auth with refresh tokens" \
     --content "Used JWT for stateless auth. Access token 15min, refresh 7d. Stored in httpOnly cookies. Chose bcrypt for password hashing."

4. If something broke during implementation:
   am mem add gotcha "Prisma client not regenerating" \
     --content "After schema changes, must run 'npx prisma generate' before 'npm run dev'. The dev server doesn't auto-regenerate."

5. Now respond to user
```

## Commands Quick Reference

```bash
# Add memory
am mem add <type> "<title>" --content "<details>"

# List existing (check before adding)
am mem list

# Search memories
am mem search "<query>"

# Sync to git
am sync

# Protect file
am protect "<path>" "<reason>"

# Register tool
am tool "<path>" "<description>" "<usage>"
```

Remember: **Unsaved memories are lost memories.** Save early, save often.
