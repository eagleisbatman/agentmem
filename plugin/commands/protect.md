---
description: Mark a file as protected - requires approval before modification
---

# Protect Command

Mark a file or pattern as protected. Protected files require explicit user approval before modification.

## Usage

Parse the user's input to extract:
1. **Path** - File path or glob pattern
2. **Reason** - Why it's protected (optional)

Run the command:
```bash
am protect "<path>" "<reason>"
```

## Examples

User: "/agentmem:protect src/config.ts"
Run: `am protect "src/config.ts" "Critical configuration file"`

User: "/agentmem:protect prisma/schema.prisma Database schema"
Run: `am protect "prisma/schema.prisma" "Database schema - changes require migration"`

User: "/agentmem:protect *.env*"
Run: `am protect "*.env*" "Environment files contain secrets"`

After running the command, confirm the file is now protected.

## Important

When you see a protected file in your context, ALWAYS ask for approval before modifying it.
