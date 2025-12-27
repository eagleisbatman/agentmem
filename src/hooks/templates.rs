// =============================================================================
// CLAUDE CODE TEMPLATES
// =============================================================================

/// Pre-prompt hook template for Claude Code
/// This hook runs before each user prompt and injects AgentMem context
pub const CLAUDE_PRE_PROMPT_HOOK: &str = r#"#!/usr/bin/env node
const { execSync } = require('child_process');

// Read input from stdin
let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    const event = JSON.parse(input);
    const prompt = event.prompt || '';

    // Escape the prompt for shell
    const escapedPrompt = prompt.replace(/"/g, '\\"').replace(/\n/g, ' ');

    // Get context from AgentMem CLI
    const result = execSync(
      `am context --query "${escapedPrompt}" --json`,
      { encoding: 'utf-8', timeout: 5000 }
    );

    const context = JSON.parse(result);

    // Format as markdown
    let injection = '';

    if (context.protected && context.protected.length > 0) {
      injection += '## Protected Files\n';
      injection += 'Ask before modifying:\n';
      context.protected.forEach(p => {
        injection += `- \`${p.pattern}\` - ${p.reason || 'No reason provided'}\n`;
      });
      injection += '\n';
    }

    if (context.tasks && context.tasks.length > 0) {
      injection += '## Current Tasks\n';
      context.tasks.forEach(t => {
        injection += `- [P${t.priority}] ${t.id}: ${t.title} (${t.status})\n`;
      });
      injection += '\n';
    }

    if (context.memories && context.memories.length > 0) {
      injection += '## Relevant Context\n';
      context.memories.forEach(m => {
        injection += `- [${m.memory_type}] ${m.title}: ${m.content || ''}\n`;
      });
      injection += '\n';
    }

    if (context.tools && context.tools.length > 0) {
      injection += '## Available Tools\n';
      context.tools.forEach(t => {
        injection += `- \`${t.location}\` - ${t.description || ''}\n`;
      });
    }

    if (injection.trim()) {
      console.log(JSON.stringify({
        contextPrefix: `\n---\n${injection}---\n`
      }));
    } else {
      console.log(JSON.stringify({}));
    }

  } catch (error) {
    // Silently fail - output empty result
    console.log(JSON.stringify({}));
  }
});
"#;

/// Session start hook template for Claude Code
/// This hook runs when a session starts to register it with the cloud
pub const CLAUDE_SESSION_START_HOOK: &str = r#"#!/usr/bin/env node
const { execSync } = require('child_process');

// Read input from stdin
let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    const event = JSON.parse(input);
    const model = event.model || '';

    // Start cloud session tracking
    execSync(
      `am session start --agent claude-code${model ? ` --model "${model}"` : ''}`,
      { encoding: 'utf-8', timeout: 5000 }
    );

    console.log(JSON.stringify({}));
  } catch (error) {
    // Silently fail
    console.log(JSON.stringify({}));
  }
});
"#;

/// Post-session hook template for Claude Code
/// This hook runs after session end for memory extraction and sync
pub const CLAUDE_POST_SESSION_HOOK: &str = r#"#!/usr/bin/env node
const { spawn, execSync } = require('child_process');
const fs = require('fs');
const os = require('os');
const path = require('path');

// Read input from stdin
let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    const event = JSON.parse(input);
    const { transcript, sessionId, tokensIn, tokensOut, model } = event;

    // End cloud session tracking
    try {
      let cmd = 'am session end';
      if (tokensIn) cmd += ` --tokens-in ${tokensIn}`;
      if (tokensOut) cmd += ` --tokens-out ${tokensOut}`;
      if (model) cmd += ` --model "${model}"`;
      execSync(cmd, { encoding: 'utf-8', timeout: 5000 });
    } catch (e) {
      // Silently continue
    }

    // Write transcript to temp file if provided
    if (transcript && Array.isArray(transcript) && transcript.length > 0) {
      const tempDir = os.tmpdir();
      const transcriptFile = path.join(tempDir, `am-transcript-${sessionId || Date.now()}.jsonl`);

      // Write transcript as JSONL
      const content = transcript
        .map(msg => JSON.stringify(msg))
        .join('\n');
      fs.writeFileSync(transcriptFile, content);

      // Extract memories (non-blocking)
      spawn('am', ['extract', '--transcript', transcriptFile], {
        detached: true,
        stdio: 'ignore'
      }).unref();

      // Clean up temp file after a delay
      setTimeout(() => {
        try { fs.unlinkSync(transcriptFile); } catch (e) {}
      }, 60000);
    }

    // Sync to git (non-blocking)
    spawn('am', ['sync'], {
      detached: true,
      stdio: 'ignore'
    }).unref();

    // Output success
    console.log(JSON.stringify({}));

  } catch (error) {
    // Silently fail
    console.log(JSON.stringify({}));
  }
});
"#;

/// CLAUDE.md section to append when installing hooks
pub const CLAUDE_MD_SECTION: &str = r#"
## AgentMem Integration

This project uses AgentMem for persistent context across sessions.

**Quick commands:**
- `am task ready` - See unblocked tasks
- `am context` - Get relevant memories
- `am mem add <type> <title>` - Add a memory
- `am protect <file>` - Mark file as protected
- `am sync` - Sync to git

Protected files require approval before modification.
"#;

// =============================================================================
// GEMINI CLI TEMPLATES
// =============================================================================

/// Pre-tool hook template for Gemini CLI
/// Gemini CLI uses stdin/stdout JSON for hooks
pub const GEMINI_PRE_TOOL_HOOK: &str = r#"#!/usr/bin/env node
const { execSync } = require('child_process');

// Read JSON input from stdin
let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    const event = JSON.parse(input);

    // Get context from AgentMem
    const result = execSync('am context --json', { encoding: 'utf-8', timeout: 5000 });
    const context = JSON.parse(result);

    // Output context as JSON (Gemini will include in prompt)
    const output = {
      status: 'continue',
      context: formatContext(context)
    };

    console.log(JSON.stringify(output));
  } catch (error) {
    // Don't block on errors
    console.log(JSON.stringify({ status: 'continue' }));
  }
});

function formatContext(context) {
  let text = '';

  if (context.protected?.length > 0) {
    text += '## Protected Files\n';
    context.protected.forEach(p => {
      text += `- \`${p.pattern}\` - ${p.reason || 'Protected'}\n`;
    });
    text += '\n';
  }

  if (context.memories?.length > 0) {
    text += '## Relevant Context\n';
    context.memories.forEach(m => {
      text += `- [${m.memory_type}] ${m.title}: ${m.content || ''}\n`;
    });
  }

  return text;
}
"#;

/// Session end hook for Gemini CLI
pub const GEMINI_SESSION_END_HOOK: &str = r#"#!/usr/bin/env node
const { spawn } = require('child_process');

// Read JSON input from stdin
let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => input += chunk);
process.stdin.on('end', () => {
  try {
    // Sync to git (non-blocking)
    spawn('am', ['sync'], {
      detached: true,
      stdio: 'ignore'
    }).unref();

    console.log(JSON.stringify({ status: 'success' }));
  } catch (error) {
    console.log(JSON.stringify({ status: 'success' }));
  }
});
"#;

/// GEMINI.md context file content
pub const GEMINI_MD_CONTENT: &str = r#"# AgentMem Context

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
"#;

// =============================================================================
// CODEX CLI TEMPLATES
// =============================================================================

/// Wrapper script for Codex CLI that injects AgentMem context
pub const CODEX_WRAPPER_SCRIPT: &str = r#"#!/usr/bin/env bash
#
# AgentMem wrapper for Codex CLI
# Usage: am-codex [codex args...]
#

# Get AgentMem context and add to prompt
CONTEXT=$(am context 2>/dev/null)

if [ -n "$CONTEXT" ]; then
  echo "--- AgentMem Context ---"
  echo "$CONTEXT"
  echo "------------------------"
  echo ""
fi

# Run codex with all arguments
exec codex "$@"
"#;

/// Instructions file for Codex CLI (placed in project root)
pub const CODEX_INSTRUCTIONS: &str = r#"# AgentMem Integration

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
"#;

// =============================================================================
// CURSOR TEMPLATES
// =============================================================================

/// MDC rule file for Cursor (.cursor/rules/agentmem.mdc)
pub const CURSOR_MDC_RULE: &str = r#"---
description: AgentMem persistent context integration
globs: ["**/*"]
alwaysApply: true
---

# AgentMem Integration

This project uses AgentMem for persistent memory across AI sessions.

## Before Making Changes

Run `am context` in the terminal to check for:
- Protected files that require approval before modification
- Relevant memories about decisions, patterns, and gotchas
- Current tasks and their priorities

## Protected Files

Always check protected files before modifying. Run:
```
am context --query "protected"
```

If a file is protected, ask the user for explicit approval before making changes.

## Adding Memories

When you learn something important, suggest adding it:
- Corrections: `am mem add correction "Title" --content "What was wrong"`
- Decisions: `am mem add decision "Title" --content "Why this choice"`
- Gotchas: `am mem add gotcha "Title" --content "What to watch out for"`

## Commands Reference

| Command | Description |
|---------|-------------|
| `am context` | Get relevant memories |
| `am context --query "search"` | Search memories |
| `am mem add <type> <title>` | Add a memory |
| `am protect <file>` | Mark file as protected |
| `am task ready` | See unblocked tasks |
| `am sync` | Sync to git |
"#;

/// Legacy .cursorrules file (for backward compatibility)
pub const CURSOR_RULES_LEGACY: &str = r#"# AgentMem Integration

This project uses AgentMem for persistent memory across AI sessions.

## Important Rules

1. Before modifying any file, run `am context` to check if it's protected
2. Protected files require explicit user approval before changes
3. Check for relevant memories about the current task

## Commands
- `am context` - Get relevant context
- `am context --query "search term"` - Search memories
- `am mem add <type> <title>` - Add a memory
- `am protect <file>` - Mark file as protected
- `am sync` - Sync to git

## Memory Types
- correction: Agent mistakes to avoid
- decision: Technical/architectural choices
- gotcha: Things that broke or surprised
- pattern: Repeated behaviors
- infrastructure: URLs, endpoints, configs
"#;

// =============================================================================
// BACKWARDS COMPATIBILITY (deprecated, use agent-specific versions)
// =============================================================================

/// Legacy pre-prompt hook (alias for Claude Code)
pub const PRE_PROMPT_HOOK: &str = CLAUDE_PRE_PROMPT_HOOK;

/// Legacy post-session hook (alias for Claude Code)
pub const POST_SESSION_HOOK: &str = CLAUDE_POST_SESSION_HOOK;
