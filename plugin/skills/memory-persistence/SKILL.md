---
name: memory-persistence
description: Automatically persist session state and extract learnings. Use when you notice decisions, corrections, gotchas, or at session end to preserve knowledge.
---

# Memory Persistence Skill

You have access to AgentMem for persistent memory across sessions. Use this skill to preserve important learnings automatically.

## When to Trigger

### 1. User Corrections
When the user corrects you with phrases like:
- "No, use X instead"
- "Actually, it should be..."
- "That's wrong, do it this way"
- "Don't do that"

**Action**: Run `am mem add correction "<what you learned>" --content "<details>"`

### 2. Decisions Made
When a decision is reached about:
- Technology choices ("Let's use PostgreSQL")
- Architecture patterns ("We'll use microservices")
- Conventions ("Use camelCase for variables")

**Action**: Run `am mem add decision "<decision>" --content "<reasoning>"`

### 3. Things That Broke (Gotchas)
When something fails unexpectedly:
- Build errors with non-obvious causes
- Runtime issues
- Configuration problems
- API quirks

**Action**: Run `am mem add gotcha "<what broke>" --content "<why and how to fix>"`

### 4. Infrastructure Details
When you learn about:
- API endpoints
- Database connections
- Environment variables
- Service URLs

**Action**: Run `am mem add infrastructure "<what>" --content "<details>"`

### 5. Existing Tools/Scripts
When you discover or are told about:
- Existing utility scripts
- Build commands
- Test runners
- Deployment tools

**Action**: Run `am tool "<path>" "<description>" "<usage>"`

### 6. Protected Files
When the user indicates a file shouldn't be changed:
- "Don't modify this file"
- "This is working, leave it alone"
- Config files that are sensitive

**Action**: Run `am protect "<file>" "<reason>"`

## Session End Behavior

When the session is ending or you detect context compaction:

1. Review the conversation for any unrecorded learnings
2. Run `am mem add` for each important learning
3. Run `am sync` to persist to git

## TodoWrite State Persistence

When you use the TodoWrite tool to track tasks:

1. The current state is automatically captured by hooks
2. At session end, the state is saved to AgentMem
3. Next session, the pre-prompt hook restores your task context

You don't need to manually save TodoWrite state - just use it normally.

## Best Practices

- Be proactive: Don't wait to be asked to save memories
- Be concise: Titles should be 5-10 words
- Include context: The content field should explain "why" not just "what"
- Avoid noise: Only save genuinely useful learnings, not obvious facts
- Deduplicate: Before adding, consider if you already know this

## Example Workflow

```
User: "The API endpoint is https://api.myapp.com/v2, not v1"

You detect: This is a correction about infrastructure

Run: am mem add correction "API is v2 not v1" --content "Production API endpoint is https://api.myapp.com/v2. The v1 endpoint is deprecated."

Report: "I've saved that the API uses v2. I won't make that mistake again."
```
