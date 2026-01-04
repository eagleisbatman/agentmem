---
description: Add a memory to AgentMem. Types: decision, correction, gotcha, pattern, infrastructure, tool, protected, insight
---

# Remember Command

Add a memory to AgentMem for persistent context across sessions.

## Usage

The user has requested to remember something. Parse their input to extract:
1. **Type** - One of: decision, correction, gotcha, pattern, infrastructure, tool, protected, insight
2. **Title** - A short descriptive title (5-10 words)
3. **Content** - The full details (optional, from context)

Run the command:
```bash
am mem add <type> "<title>" --content "<details>"
```

## Memory Types

| Type | When to Use |
|------|-------------|
| `decision` | Architectural or technical choices with reasoning |
| `correction` | When the user corrected a mistake |
| `gotcha` | Things that broke or surprised |
| `pattern` | Repeated preferences or behaviors |
| `infrastructure` | URLs, endpoints, API details |
| `tool` | Scripts or utilities to use |
| `protected` | Files not to modify |
| `insight` | Non-obvious discoveries |

## Examples

User: "/agentmem:remember decision Use PostgreSQL for JSON support"
Run: `am mem add decision "Use PostgreSQL" --content "Chose PostgreSQL over MySQL for native JSON support and better performance"`

User: "/agentmem:remember gotcha Safari breaks with SameSite cookies"
Run: `am mem add gotcha "Safari SameSite cookie issue" --content "SameSite=Lax cookies break authentication on Safari. Use SameSite=None with Secure flag."`

After running the command, confirm what was saved.
